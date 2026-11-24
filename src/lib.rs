pub mod application;
pub mod rendering;
pub mod utils;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::{cell::RefCell, rc::Rc};

    use wasm_bindgen::prelude::*;
    use winit::{
        event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        keyboard::{KeyCode, PhysicalKey},
        platform::web::WindowExtWebSys,
        window::Window,
    };

    use crate::{
        application::state::State,
        rendering::{camera::Direction, light::Light, sphere::Sphere},
    };

    struct WasmApp {
        window: Option<std::sync::Arc<Window>>,
        state: Rc<RefCell<Option<State<'static>>>>,
        movements: [bool; 3],
        last_mouse_pos: winit::dpi::PhysicalPosition<f64>,
    }

    impl Default for WasmApp {
        fn default() -> Self {
            Self {
                window: None,
                state: Rc::new(RefCell::new(None)),
                movements: [false; 3],
                last_mouse_pos: winit::dpi::PhysicalPosition::new(0.0, 0.0),
            }
        }
    }

    impl winit::application::ApplicationHandler for WasmApp {
        fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        }

        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }
            let window = std::sync::Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            window.set_title("3D Renderer");

            let canvas = window.canvas().unwrap();
            let web_window = web_sys::window().unwrap();
            let document = web_window.document().unwrap();
            let body = document.body().unwrap();
            let css_w = web_window.inner_width().unwrap().as_f64().unwrap_or(800.0);
            let css_h = web_window.inner_height().unwrap().as_f64().unwrap_or(600.0);
            // Render at reduced internal resolution; CSS stretches the canvas to fill viewport.
            let render_scale: f64 = 0.5;
            let w = ((css_w * render_scale) as u32).max(1);
            let h = ((css_h * render_scale) as u32).max(1);
            canvas.set_width(w);
            canvas.set_height(h);
            let style = canvas.style();
            let _ = style.set_property("width", &format!("{}px", css_w as u32));
            let _ = style.set_property("height", &format!("{}px", css_h as u32));
            body.append_child(&canvas).unwrap();
            log::info!("Canvas internal: {}x{} | CSS: {}x{}", w, h, css_w, css_h);
            let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(w, h));

            let state_holder = self.state.clone();
            let window_clone = window.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut state = State::new(window_clone, None).await;

                state.cam_manager.camera.position.v[2] = -18.;
                state.cam_manager.camera.near = 0.001;

                // Sky is bright at +Y (sky color), dark at -Y (horizon black).
                // Place lights up high so the lit side of objects matches the bright
                // part of the sky, leaving the bottom-facing side naturally dark.
                let lights = vec![
                    // Main sun-like light directly overhead, tinted slightly toward sky blue
                    Light {
                        position: [0., 120., 20.],
                        is_valid: 1,
                        color: [0.95, 0.97, 1.0, 1.0],
                        intensity: 25.,
                        ..Default::default()
                    },
                    // Soft fill from the upper-front to lift shadowed sides a little
                    Light {
                        position: [80., 60., -40.],
                        is_valid: 1,
                        color: [0.7, 0.8, 1.0, 1.0],
                        intensity: 10.,
                        ..Default::default()
                    },
                ];
                state.light_manager.add_lights(lights, &state.device, &state.queue);

                let spheres = vec![
                    Sphere { center: [-3., 2., 5.], radius: 1.5, color: [0.9, 0.2, 0.2, 1.0], refractivity: 1.5, material: 0.0, ..Default::default() },
                    Sphere { center: [3., -1., 3.], radius: 1.0, color: [0.2, 0.6, 0.9, 1.0], refractivity: 1.0, material: 1.0, ..Default::default() },
                    Sphere { center: [0., 4., 8.], radius: 2.0, color: [0.3, 0.9, 0.3, 1.0], refractivity: 2.0, material: 0.5, ..Default::default() },
                ];
                state.sphere_manager.add_spheres(spheres, &state.device, &state.queue);

                state.cam_manager.camera.update_cam_info(&state.size);
                state.queue.write_buffer(
                    &state.cam_manager.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[state.cam_manager.camera.cam_info]),
                );

                log::info!("3D Renderer state initialised");
                *state_holder.borrow_mut() = Some(state);
            });

            self.window = Some(window);
        }

        fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
            if self.window.as_ref().map(|w| w.id()) != Some(window_id) {
                return;
            }

            match &event {
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state: ElementState::Pressed,
                    ..
                } => { self.movements[0] = true; return; }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state: ElementState::Released,
                    ..
                } => { self.movements[0] = false; return; }
                WindowEvent::MouseInput {
                    button: MouseButton::Right,
                    state: ElementState::Pressed,
                    ..
                } => { self.movements[1] = true; return; }
                WindowEvent::MouseInput {
                    button: MouseButton::Right,
                    state: ElementState::Released,
                    ..
                } => { self.movements[1] = false; return; }
                WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::KeyQ)
                            | PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                    ..
                } => { event_loop.exit(); return; }
                WindowEvent::CloseRequested => { event_loop.exit(); return; }
                _ => {}
            }

            let mut st = self.state.borrow_mut();
            let Some(state) = st.as_mut() else { return; };

            match event {
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                    state.cam_manager.camera.update_cam_info(&physical_size);
                    state.queue.write_buffer(
                        &state.cam_manager.camera_buffer,
                        0,
                        bytemuck::cast_slice(&[state.cam_manager.camera.cam_info]),
                    );
                    let _ = state.render();
                }
                WindowEvent::RedrawRequested => {
                    let _ = state.render();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if self.movements[0] || self.movements[1] {
                        let x_diff = self.last_mouse_pos.x - position.x;
                        let y_diff = self.last_mouse_pos.y - position.y;
                        if x_diff > 0. {
                            state.cam_manager.camera.movement(Direction::Left, &self.movements[0], x_diff.abs() as f32);
                        } else if x_diff < 0. {
                            state.cam_manager.camera.movement(Direction::Right, &self.movements[0], x_diff.abs() as f32);
                        }
                        if y_diff > 0. {
                            state.cam_manager.camera.movement(Direction::Up, &self.movements[0], y_diff.abs() as f32);
                        } else if y_diff < 0. {
                            state.cam_manager.camera.movement(Direction::Down, &self.movements[0], y_diff.abs() as f32);
                        }
                        state.cam_manager.camera.update_cam_info(&state.size);
                        state.queue.write_buffer(
                            &state.cam_manager.camera_buffer,
                            0,
                            bytemuck::cast_slice(&[state.cam_manager.camera.cam_info]),
                        );
                        let _ = state.render();
                    }
                    self.last_mouse_pos = position;
                }
                WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_x, y), .. } => {
                    let forward = (&state.cam_manager.camera.focus - &state.cam_manager.camera.position).normalize();
                    if y > 0. {
                        state.cam_manager.camera.position += forward * 1.01;
                    } else {
                        state.cam_manager.camera.position -= forward * 0.99;
                    }
                    state.cam_manager.camera.update_cam_info(&state.size);
                    state.queue.write_buffer(
                        &state.cam_manager.camera_buffer,
                        0,
                        bytemuck::cast_slice(&[state.cam_manager.camera.cam_info]),
                    );
                    let _ = state.render();
                }
                WindowEvent::KeyboardInput { event: KeyEvent { physical_key, .. }, .. } => {
                    match physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            state.cam_manager.camera.movement(Direction::Forward, &self.movements[0], 2.5);
                        }
                        PhysicalKey::Code(KeyCode::KeyS) => {
                            state.cam_manager.camera.movement(Direction::Backward, &self.movements[0], 2.5);
                        }
                        PhysicalKey::Code(KeyCode::KeyA) => {
                            state.cam_manager.camera.movement(Direction::Left, &self.movements[0], 2.5);
                        }
                        PhysicalKey::Code(KeyCode::KeyD) => {
                            state.cam_manager.camera.movement(Direction::Right, &self.movements[0], 2.5);
                        }
                        PhysicalKey::Code(KeyCode::KeyZ) => {
                            state.cam_manager.camera.movement(Direction::Up, &self.movements[0], 2.5);
                        }
                        PhysicalKey::Code(KeyCode::KeyX) => {
                            state.cam_manager.camera.movement(Direction::Down, &self.movements[0], 2.5);
                        }
                        _ => return,
                    }
                    state.cam_manager.camera.update_cam_info(&state.size);
                    state.queue.write_buffer(
                        &state.cam_manager.camera_buffer,
                        0,
                        bytemuck::cast_slice(&[state.cam_manager.camera.cam_info]),
                    );
                    let _ = state.render();
                }
                _ => {}
            }
        }
    }

    #[wasm_bindgen(start)]
    pub fn wasm_main() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("Could not initialize logger");

        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = WasmApp::default();
        event_loop.run_app(&mut app).unwrap();
    }
}
