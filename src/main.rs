use std::f32::consts::PI;
use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use winit::dpi::{PhysicalPosition, PhysicalSize};

pub mod ray;
pub mod vector;
pub mod utils;

use crate::vector::Vec3;
use crate::ray::Camera;

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0]}, // Bottom-left
    Vertex { position: [1.0, -1.0, 0.0]},  // Bottom-right
    Vertex { position: [-1.0, 1.0, 0.0]},  // Top-left
    Vertex { position: [1.0, 1.0, 0.0]},   // Top-right
];

const INDICES: &[u16] = &[
    0, 1, 2,  // First triangle
    1, 3, 2,  // Second triangle
];


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::NoUninit)]
struct CamInfo {
    cam_info: [[f32; 4]; 4],
    //[cam_pos[0], cam_pos[1], cam_pos[2], view_port_center[0]]
    //[view_port_center[1], view_port_center[2], pixel_width, pixel_height]
    //[half_width, half_height, x[0], x[1]]
    //[x[2], y[0], y[1], y[2]]
}

impl CamInfo {
    fn update_self(&mut self, camera: &Camera, size: &PhysicalSize<u32>){
        
        let aspect_ratio = size.width as f32 / size.height as f32;
        self.cam_info[0][0] = camera.position.v[0];
        self.cam_info[0][1] = camera.position.v[1];
        self.cam_info[0][2] = camera.position.v[2];
        self.cam_info[2][1] = (camera.view_angle / 2.).tan() * camera.near;
        self.cam_info[2][0] = aspect_ratio * self.cam_info[2][1];
        let z = (&camera.focus - &camera.position).normalize();
        let x = camera.up.cross(&z).normalize();
        self.cam_info[2][2] = x.v[0];
        self.cam_info[2][3] = x.v[1];
        self.cam_info[3][0] = x.v[2];
        let y = z.cross(&x).normalize();
        self.cam_info[3][1] = y.v[0];
        self.cam_info[3][2] = y.v[1];
        self.cam_info[3][3] = y.v[2];
        
        let view_port_center = &camera.position + z * camera.near;
        self.cam_info[0][3] = view_port_center.v[0];
        self.cam_info[1][0] = view_port_center.v[1];
        self.cam_info[1][1] = view_port_center.v[2];
    }
}

struct State<'a>{
    camera: Camera,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    mouse_uniform: wgpu::Buffer,
    mouse_bind_group: wgpu::BindGroup,
    cam_info: CamInfo,
    cam_info_buffer: wgpu::Buffer,
    cam_info_group: wgpu::BindGroup,
}

impl<'a> State<'a>{
    async fn new(window: Arc<Window>) -> Self {
        let camera = Camera::new(Some(Vec3::new(0., 0., -28.)),Some(Vec3::new(0., 1., 0.)),
    Some(Vec3{v:[0., 0., 0.]}), Some(0.001), None, Some(35. * (PI / 180.)));
        let size = window.inner_size();
        let instance_descriptor = wgpu::InstanceDescriptor{
            backends: wgpu::Backends::VULKAN, ..Default::default()
        };
        let instance = wgpu::Instance::new(instance_descriptor);
        let surface = instance.create_surface(window).unwrap();
        let adapter_descriptor = wgpu::RequestAdapterOptionsBase{
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();
        let device_descriptor = wgpu::DeviceDescriptor{
            memory_hints: wgpu::MemoryHints::Performance,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
        };
        let (device, queue) = adapter.request_device(&device_descriptor, None).await.unwrap();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter().copied().filter(|f| f.is_srgb()).next().unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration{
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });        
        let mouse_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mouse Uniform Buffer"),
            size: std::mem::size_of::<[f32; 2]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mouse_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Mouse Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let mouse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mouse Bind Group"),
            layout: &mouse_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: mouse_uniform.as_entire_binding(),
                },
            ],
        });
        let aspect_ratio = size.width as f32 / size.height as f32;
        let half_height = (camera.view_angle / 2.).tan() * camera.near;
        let half_width = aspect_ratio * half_height;
        let z = (&camera.focus - &camera.position).normalize();
        let x = camera.up.cross(&z).normalize();
        let y = z.cross(&x).normalize();
        
        let pixel_width = 2. * half_width / size.width as f32;
        let pixel_height = 2. * half_height / size.height as f32;
        let view_port_center = &camera.position + z * camera.near;
        let cam_info = CamInfo{
            cam_info: [
                [camera.position.v[0], camera.position.v[1], camera.position.v[2], view_port_center.v[0]],
                [view_port_center.v[1], view_port_center.v[2], pixel_width, pixel_height],
                [half_width, half_height, x.v[0], x.v[1]],
                [x.v[2], y.v[0], y.v[1], y.v[2]]]
        };
        let cam_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Info Uniform Buffer"),
            contents: bytemuck::cast_slice(&[cam_info]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let cam_info_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("CamInfo Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });
        let cam_info_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cam_info Bind Group"),
            layout: &cam_info_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cam_info_buffer.as_entire_binding(),
                },
            ],
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&mouse_bind_group_layout, &cam_info_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None, // 6.
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
        });
        surface.configure(&device, &config);
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;
         
        Self {
            camera,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer, 
            num_indices,
            mouse_bind_group,
            mouse_uniform,
            cam_info,
            cam_info_buffer,
            cam_info_group: cam_info_bind_group,
        }  
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>){
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device,  &self.config);
        }
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>{
        let drawable = self.surface.get_current_texture()?;
        let image_view = drawable.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder")
        });
        {
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &image_view,
                resolve_target: None,
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            };

            let render_passs_descriptor = wgpu::RenderPassDescriptor{
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None
            };

            let mut render_pass = command_encoder.begin_render_pass(&render_passs_descriptor);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.mouse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.cam_info_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 2.
        }
        self.queue.submit(std::iter::once(command_encoder.finish()));
        drawable.present();
        Ok(())
    }
}

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    state: Option<State<'a>>,
    ctrl_state: bool,
    mouse_state: [bool ; 3],
    last_mouse_pos: PhysicalPosition<f64>
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("App resumed");
        if self.window.is_none() {
            let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
            self.window = Some(window.clone());

            let state = pollster::block_on(State::new(window.clone()));
            self.state = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if id != self.window.as_ref().unwrap().id() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::Resized(physical_size) => {
                self.state.as_mut().unwrap().resize(physical_size);
                let size = self.state.as_ref().unwrap().size;
                let camera = &self.state.as_ref().unwrap().camera;
                let aspect_ratio = size.width as f32 / size.height as f32;
                let half_height = (camera.view_angle / 2.).tan() * camera.near;
                let half_width = aspect_ratio * half_height;
                let pixel_width = 2. * half_width / size.width as f32;
                let pixel_height = 2. * half_height / size.height as f32;
                let cam_info = &mut self.state.as_mut().unwrap().cam_info;
                cam_info.cam_info[1][2] = pixel_width;
                cam_info.cam_info[1][3] = pixel_height;
                cam_info.cam_info[2][0] = half_width;
                cam_info.cam_info[2][1] = half_height;
                self.state.as_ref().unwrap().queue.write_buffer(&self.state.as_ref().unwrap().cam_info_buffer, 0, bytemuck::cast_slice(&[self.state.as_ref().unwrap().cam_info]));
                let _ = self.state.as_mut().unwrap().render();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                // let _ = self.state.as_mut().unwrap().render();
            },
            WindowEvent::KeyboardInput { event: KeyEvent{
                state: ElementState::Pressed,
                physical_key: PhysicalKey::Code(KeyCode::KeyQ) | PhysicalKey::Code(KeyCode::Escape),
                ..
            }, ..} => event_loop.exit(),
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                ..
            } => {
                    self.mouse_state[0] = true;
            },
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                ..
            } => {
                    self.mouse_state[0] = false;
            },
            WindowEvent::MouseInput {
                button: MouseButton::Middle,
                state: ElementState::Pressed,
                ..
            } => {
                    self.mouse_state[1] = true;
            },
            WindowEvent::MouseInput {
                button: MouseButton::Middle,
                state: ElementState::Released,
                ..
            } => {
                    self.mouse_state[1] = false;
            },
            WindowEvent::CursorMoved {position, .. } => {
                // let size = self.state.as_ref().unwrap().size;
                // let mouse_pos = [
                //     position.x as f32 / size.width as f32 * 2.0 - 1.0,
                //     1.0 - position.y as f32 / size.height as f32 * 2.0,
                // ];
                // self.state.as_ref().unwrap().queue.write_buffer(&self.state.as_ref().unwrap().mouse_uniform, 0, bytemuck::cast_slice(&mouse_pos));
                if self.mouse_state[0] || self.mouse_state[1] {
                    let x_diff = self.last_mouse_pos.x - position.x;
                    let y_diff = self.last_mouse_pos.y - position.y;
                    let state = self.state.as_mut().unwrap();
                    if x_diff > 0. {
                        state.camera.movement(ray::Direction::Left, &self.mouse_state[0]);
                    }
                    else if x_diff < 0. {
                        state.camera.movement(ray::Direction::Right, &self.mouse_state[0]);
                    }
                    if y_diff > 0. {
                        state.camera.movement(ray::Direction::Up, &self.mouse_state[0]);
                    }
                    else if y_diff < 0. {
                        state.camera.movement(ray::Direction::Down, &self.mouse_state[0]);
                    }
                    state.cam_info.update_self(&state.camera, &state.size);
                    state.queue.write_buffer(&state.cam_info_buffer, 0, bytemuck::cast_slice(&[state.cam_info]));
                    let _ = state.render();
                }
                self.last_mouse_pos = position;
            },
            WindowEvent::CursorLeft { .. } => {
                let mouse_pos: [f32; 2] = [
                    2.,
                    2.,
                ];
                self.state.as_ref().unwrap().queue.write_buffer(&self.state.as_ref().unwrap().mouse_uniform, 0, bytemuck::cast_slice(&mouse_pos));
                let _ = self.state.as_mut().unwrap().render();
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { 
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::ControlLeft),
                    .. 
                },
                ..
            } => {
                self.ctrl_state = true;
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { 
                    state: ElementState::Released,
                    physical_key: PhysicalKey::Code(KeyCode::ControlLeft),
                    .. 
                },
                ..
            } => {
                self.ctrl_state = false;
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { 
                    physical_key: PhysicalKey::Code(KeyCode::KeyW),
                    .. 
                },
                ..
            } => {
                {
                    let state = self.state.as_mut().unwrap();
                    state.camera.movement(ray::Direction::Forward, &self.ctrl_state);
                    state.cam_info.update_self(&state.camera, &state.size);
                }
                let state = self.state.as_ref().unwrap();
                state.queue.write_buffer(&state.cam_info_buffer, 0, bytemuck::cast_slice(&[self.state.as_ref().unwrap().cam_info]));
                let _ = self.state.as_mut().unwrap().render();
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { 
                    physical_key: PhysicalKey::Code(KeyCode::KeyS),
                    ..
                },
                ..
            } => {
                {
                    let state = self.state.as_mut().unwrap();
                    state.camera.movement(ray::Direction::Backward, &self.ctrl_state);
                    state.cam_info.update_self(&state.camera, &state.size);
                }
                let state = self.state.as_ref().unwrap();
                state.queue.write_buffer(&state.cam_info_buffer, 0, bytemuck::cast_slice(&[self.state.as_ref().unwrap().cam_info]));
                let _ = self.state.as_mut().unwrap().render();
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { 
                    physical_key: PhysicalKey::Code(KeyCode::KeyD),
                    ..
                },
                ..
            } => {
                {
                    let state = self.state.as_mut().unwrap();
                    state.camera.movement(ray::Direction::Right, &self.ctrl_state);
                    state.cam_info.update_self(&state.camera, &state.size);
                }
                let state = self.state.as_ref().unwrap();
                state.queue.write_buffer(&state.cam_info_buffer, 0, bytemuck::cast_slice(&[self.state.as_ref().unwrap().cam_info]));
                let _ = self.state.as_mut().unwrap().render();
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { 
                    physical_key: PhysicalKey::Code(KeyCode::KeyA),
                    ..
                },
                ..
            } => {
                {
                    let state = self.state.as_mut().unwrap();
                    state.camera.movement(ray::Direction::Left, &self.ctrl_state);
                    state.cam_info.update_self(&state.camera, &state.size);
                }
                let state = self.state.as_ref().unwrap();
                state.queue.write_buffer(&state.cam_info_buffer, 0, bytemuck::cast_slice(&[self.state.as_ref().unwrap().cam_info]));
                let _ = self.state.as_mut().unwrap().render();
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { 
                    physical_key: PhysicalKey::Code(KeyCode::KeyZ),
                    ..
                },
                ..
            } => {
                {
                    let state = self.state.as_mut().unwrap();
                    state.camera.movement(ray::Direction::Up, &self.ctrl_state);
                    state.cam_info.update_self(&state.camera, &state.size);
                }
                let state = self.state.as_ref().unwrap();
                state.queue.write_buffer(&state.cam_info_buffer, 0, bytemuck::cast_slice(&[self.state.as_ref().unwrap().cam_info]));
                let _ = self.state.as_mut().unwrap().render();
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { 
                    physical_key: PhysicalKey::Code(KeyCode::KeyX),
                    ..
                },
                ..
            } => {
                {
                    let state = self.state.as_mut().unwrap();
                    state.camera.movement(ray::Direction::Down, &self.ctrl_state);
                    state.cam_info.update_self(&state.camera, &state.size);
                }
                let state = self.state.as_ref().unwrap();
                state.queue.write_buffer(&state.cam_info_buffer, 0, bytemuck::cast_slice(&[self.state.as_ref().unwrap().cam_info]));
                let _ = self.state.as_mut().unwrap().render();
            },
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    // let event_proxy = event_loop.create_proxy();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}