
pub struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub cam_manager: crate::rendering::camera::CamManager,
    pub sphere_manager: crate::rendering::sphere::SphereManager,
    pub light_manager: crate::rendering::light::LightManager,
    pub render_pipeline: wgpu::RenderPipeline,
}


impl<'a> State<'a>{
    pub async fn new(window: std::sync::Arc<winit::window::Window>,  camera: Option<crate::rendering::camera::Camera>) -> Self{
        // Creating a default camera for perspective view if default not provided by the user
        let camera = camera.unwrap_or_default();
        let window_size = window.inner_size();

        // creating instance to interact with the gpu
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor{
                backends: wgpu::Backends::VULKAN, ..Default::default()
            }
        );
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter( // To define what type of gpu to use for rendering
            &wgpu::RequestAdapterOptionsBase{
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                memory_hints: wgpu::MemoryHints::Performance,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            }, None).await.unwrap();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor{
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shader/shader.wgsl").into()),
        });
        
        let cam_manager = crate::rendering::camera::CamManager::new(
            &device, &queue, camera, &window_size
        );

        let sphere_manager = crate::rendering::sphere::SphereManager::new(
            &device, &queue, vec![crate::rendering::sphere::Sphere::default()]
        );

        let light_manager = crate::rendering::light::LightManager::new(
            &device, &queue, vec![crate::rendering::light::Light{
                position: [-10., 10., 0.],
                is_valid: 0, 
                ..Default::default()
            }]);

        let render_pipeline_layout = 
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
                label: None,
                bind_group_layouts: &[&cam_manager.bind_group_layout, &sphere_manager.bind_group_layout, &light_manager.bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{

            label: None,

            layout: Some(&render_pipeline_layout),
            
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),

            depth_stencil: None,

            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            
            multiview: None,

            cache: None,

            primitive: wgpu::PrimitiveState { 
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
        });
        surface.configure(&device, &config);
        


        Self {
            surface,
            device,
            queue,
            config,
            size: window_size,
            cam_manager,
            sphere_manager,
            render_pipeline,
            light_manager  
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device,  &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>{
        let drawable = self.surface.get_current_texture()?;
        let image_view = drawable.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});
        {
            let color_attachament = wgpu::RenderPassColorAttachment {
                view: &image_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            };
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                label: None,
                color_attachments: &[Some(color_attachament)],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.cam_manager.bind_group, &[]);
            render_pass.set_bind_group(1, &self.sphere_manager.bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_manager.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        self.queue.submit(std::iter::once(command_encoder.finish()));
        drawable.present();
        Ok(())
    }
 
}
