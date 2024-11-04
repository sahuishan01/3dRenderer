use winit::{
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    application::state::State,
    rendering::{camera::Direction, light::Light, sphere::Sphere},
};
use rand::Rng;

#[derive(Default)]
pub struct App<'a> {
    pub window: Option<std::sync::Arc<Window>>,
    pub state: Option<State<'a>>,
    pub movements: [bool; 3],
    pub last_mouse_pos: winit::dpi::PhysicalPosition<f64>,
}

impl<'a> winit::application::ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        println!("App resumed");
        if self.window.is_none() {
            let window = std::sync::Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            self.window = Some(window.clone());

            let state = pollster::block_on(State::new(window, None));
            self.state = Some(state);
            let state = self.state.as_mut().unwrap();
            state.cam_manager.camera.position.v[2] = -18.;
            state.cam_manager.camera.near = 0.001;

            let lights = vec![
                Light {
                    position: [-100., -100., -100.],
                    is_valid: 1,
                    ..Default::default()
                },
                Light {
                    position: [100., 0., 100.],
                    is_valid: 1,
                    ..Default::default()
                },
            ];

            state
                .light_manager
                .add_lights(lights, &state.device, &state.queue);
            let mut rng = rand::thread_rng();
            let mut spheres: Vec<Sphere> = Vec::new();
            let range_val = 5_f32;
            for _ in 0..5 {
                let sphere = Sphere {
                    center: [
                        rng.gen_range(-range_val..range_val), // Random x position between -1000 and
                        // 1000
                        rng.gen_range(-range_val..range_val), // Random y position between -100000 and
                        // 1000
                        rng.gen_range(-range_val..range_val), // Random z position between -1000 and
                                                              // 1000
                    ],
                    radius: rng.gen_range(0.5..2.0), // Random radius between 0.5 and 5
                    color: [
                        rng.gen_range(0.0..1.0), // Random red value between 0 and 1
                        rng.gen_range(0.0..1.0), // Random green value between 0 and 1
                        rng.gen_range(0.0..1.0), // Random blue value between 0 and 1
                        1.0,                     // Alpha (opacity) is fixed at 1.0
                    ],
                    refractivity: rng.gen_range(1.0..3.0),
                    material: rng.gen_range(0.0..1.0), // Material is fixed
                    ..Default::default()               // Padding is fixed
                };

                spheres.push(sphere);
            }
            state
                .sphere_manager
                .add_spheres(spheres, &state.device, &state.queue);
        }
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if window_id != self.window.as_ref().unwrap().id() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed, stopping!");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                let state = self.state.as_mut().unwrap();
                state.resize(physical_size);
                state.cam_manager.camera.update_cam_info(&physical_size);
                state.queue.write_buffer(
                    &state.cam_manager.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[state.cam_manager.camera.cam_info]),
                );
                let _ = state.render();
            }
            WindowEvent::RedrawRequested => {}

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key:
                            PhysicalKey::Code(KeyCode::KeyQ) | PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                ..
            } => {
                self.movements[0] = true;
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                ..
            } => {
                self.movements[0] = false;
            }
            WindowEvent::MouseInput {
                button: MouseButton::Middle,
                state: ElementState::Pressed,
                ..
            } => {
                self.movements[1] = true;
            }
            WindowEvent::MouseInput {
                button: MouseButton::Middle,
                state: ElementState::Released,
                ..
            } => {
                self.movements[1] = false;
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.movements[0] || self.movements[1] {
                    let x_diff = self.last_mouse_pos.x - position.x;
                    let y_diff = self.last_mouse_pos.y - position.y;
                    let state = self.state.as_mut().unwrap();
                    if x_diff > 0. {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Left, &self.movements[0]);
                    } else if x_diff < 0. {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Right, &self.movements[0]);
                    }
                    if y_diff > 0. {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Up, &self.movements[0]);
                    } else if y_diff < 0. {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Down, &self.movements[0]);
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

            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_x, y),
                ..
            } => {
                let state = self.state.as_mut().unwrap();
                if y > 0. {
                    let forward = (&state.cam_manager.camera.focus
                        - &state.cam_manager.camera.position)
                        .normalize();
                    state.cam_manager.camera.position += forward * 1.01;
                } else {
                    let forward = (&state.cam_manager.camera.focus
                        - &state.cam_manager.camera.position)
                        .normalize();
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::ControlLeft),
                        ..
                    },
                ..
            } => {
                self.movements[0] = true;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Released,
                        physical_key: PhysicalKey::Code(KeyCode::ControlLeft),
                        ..
                    },
                ..
            } => {
                self.movements[0] = false;
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key, .. },
                ..
            } => {
                let state = self.state.as_mut().unwrap();
                match physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Forward, &self.movements[0]);
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Backward, &self.movements[0]);
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Left, &self.movements[0]);
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Right, &self.movements[0]);
                    }
                    PhysicalKey::Code(KeyCode::KeyZ) => {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Up, &self.movements[0]);
                    }
                    PhysicalKey::Code(KeyCode::KeyX) => {
                        state
                            .cam_manager
                            .camera
                            .movement(Direction::Down, &self.movements[0]);
                    }
                    _ => {
                        return;
                    }
                }
                state.cam_manager.camera.update_cam_info(&state.size);
                let state = self.state.as_ref().unwrap();
                state.queue.write_buffer(
                    &state.cam_manager.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[self
                        .state
                        .as_ref()
                        .unwrap()
                        .cam_manager
                        .camera
                        .cam_info]),
                );
                let _ = self.state.as_mut().unwrap().render();
            }
            _ => {}
        }
    }
}
