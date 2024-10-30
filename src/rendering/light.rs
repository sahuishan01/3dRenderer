use crate::utils::EntityCount;


#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::NoUninit)]
pub struct Light{
    pub position: [f32; 3],
    pub is_valid: u32,
    pub color: [f32; 4],
    pub intensity: f32,
    pub _padding: [f32; 3],
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: [0., 0., 0.],
            is_valid: 0,
            color: [1., 1., 1., 1.],
            intensity: 10.,
            _padding: [0., 0., 0.]
        }
    }
}

pub struct LightManager {
    pub light_buffer: wgpu::Buffer,
    pub count_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub lights: Vec<Light>,
}

impl LightManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, lights: Vec<Light>) -> Self {
        let (light_buffer, count_buffer, bind_group, bind_group_layout) = 
            Self::create_buffers_and_bind_group(device, &lights);

        let mut manager = Self {
            light_buffer,
            count_buffer,
            bind_group,
            bind_group_layout,
            lights,
        };

        manager.update_buffers(queue);
        manager
    }

    fn create_buffers_and_bind_group(
        device: &wgpu::Device,
        lights: &[Light],
    ) -> (wgpu::Buffer, wgpu::Buffer, wgpu::BindGroup, wgpu::BindGroupLayout) {
        let light_buffer_size = std::mem::size_of_val(lights) as wgpu::BufferAddress;
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light Buffer"),
            size: light_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light Count Buffer"),
            size: std::mem::size_of::<EntityCount>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("light Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: count_buffer.as_entire_binding(),
                },
            ],
            label: Some("light Bind Group"),
        });

        (light_buffer, count_buffer, bind_group, bind_group_layout)
    }

    pub fn update_buffers(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&self.lights));
        let count = EntityCount{ count: self.lights.len() as u32 };
        queue.write_buffer(&self.count_buffer, 0, bytemuck::cast_slice(&[count]));
    }

    pub fn add_light(&mut self, light: Light, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.lights.push(light);
        self.recreate_buffer_if_necessary(device);
        self.update_buffers(queue);
    }

    pub fn add_lights(&mut self, lights: Vec<Light>, device: &wgpu::Device, queue: &wgpu::Queue){
        self.lights.extend(lights);
        self.recreate_buffer_if_necessary(device);
        self.update_buffers(queue);
    }

    pub fn remove_light(&mut self, index: usize, queue: &wgpu::Queue) {
        if index < self.lights.len() {
            self.lights.remove(index);
            self.update_buffers(queue);
        }
    }

    pub fn recreate_buffer_if_necessary(&mut self, device: &wgpu::Device) {
        let required_size = (self.lights.len() * std::mem::size_of::<Light>()) as wgpu::BufferAddress;
        if required_size > self.light_buffer.size() { 
            let (light_buffer, new_count_buffer, new_bind_group, _) = 
                Self::create_buffers_and_bind_group(device, &self.lights);
            self.light_buffer = light_buffer;
            self.count_buffer = new_count_buffer;
            self.bind_group = new_bind_group;
        }
    }

    pub fn light_count(&self) -> u32 {
        self.lights.len() as u32
    }
}
