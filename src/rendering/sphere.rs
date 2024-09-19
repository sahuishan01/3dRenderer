use std::mem;
use crate::utils::EntityCount;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
    pub material: u32,
    pub padding_: [f32; 3]
}

impl Default for Sphere {
    fn default() -> Self {
        Sphere {
            center: [0., 0., 0.],
            radius: 0.,
            color: [1.0, 1.0, 0., 1.0],
            material: 0,
            padding_: [0., 0., 0.]
        }
    }
}

pub struct SphereManager {
    pub sphere_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub spheres: Vec<Sphere>,
    pub count_buffer: wgpu::Buffer,
}


impl SphereManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, initial_spheres: Vec<Sphere>) -> Self {
        let spheres = initial_spheres;
        let (sphere_buffer, count_buffer, bind_group, bind_group_layout) = 
            Self::create_buffers_and_bind_group(device, &spheres);

        let mut manager = Self {
            sphere_buffer,
            count_buffer,
            bind_group,
            bind_group_layout,
            spheres,
        };

        manager.update_buffers(queue);
        manager
    }

    fn create_buffers_and_bind_group(
        device: &wgpu::Device,
        spheres: &[Sphere],
    ) -> (wgpu::Buffer, wgpu::Buffer, wgpu::BindGroup, wgpu::BindGroupLayout) {
        let sphere_buffer_size = std::mem::size_of_val(spheres) as wgpu::BufferAddress;
        let sphere_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sphere Buffer"),
            size: sphere_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sphere Count Buffer"),
            size: mem::size_of::<EntityCount>() as wgpu::BufferAddress,
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
            label: Some("Sphere Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sphere_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: count_buffer.as_entire_binding(),
                },
            ],
            label: Some("Sphere Bind Group"),
        });

        (sphere_buffer, count_buffer, bind_group, bind_group_layout)
    }

    pub fn update_buffers(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.sphere_buffer, 0, bytemuck::cast_slice(&self.spheres));
        let count = EntityCount{ count: self.spheres.len() as u32 };
        queue.write_buffer(&self.count_buffer, 0, bytemuck::cast_slice(&[count]));
    }

    pub fn add_sphere(&mut self, sphere: Sphere, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.spheres.push(sphere);
        self.recreate_buffer_if_necessary(device);
        self.update_buffers(queue);
    }

    pub fn add_spheres(&mut self, spheres: Vec<Sphere>, device: &wgpu::Device, queue: &wgpu::Queue){
        self.spheres.extend(spheres);
        self.recreate_buffer_if_necessary(device);
        self.update_buffers(queue);
    }

    pub fn remove_sphere(&mut self, index: usize, queue: &wgpu::Queue) {
        if index < self.spheres.len() {
            self.spheres.remove(index);
            self.update_buffers(queue);
        }
    }

    pub fn recreate_buffer_if_necessary(&mut self, device: &wgpu::Device) {
        let required_size = (self.spheres.len() * mem::size_of::<Sphere>()) as wgpu::BufferAddress;
        if required_size > self.sphere_buffer.size() { 
            let (new_sphere_buffer, new_count_buffer, new_bind_group, _) = 
                Self::create_buffers_and_bind_group(device, &self.spheres);
            self.sphere_buffer = new_sphere_buffer;
            self.count_buffer = new_count_buffer;
            self.bind_group = new_bind_group;
        }
    }

    pub fn sphere_count(&self) -> u32 {
        self.spheres.len() as u32
    }
}
