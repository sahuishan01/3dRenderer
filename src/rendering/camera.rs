use std::f32::consts::PI;

use wgpu::util::DeviceExt;

use crate::utils::vector::Vec3;

#[derive(Clone)]
pub struct Camera {
    pub position: Vec3<f32>,
    pub up: Vec3<f32>,
    pub focus: Vec3<f32>,
    pub near: f32,
    pub far: f32,
    pub view_angle: f32,
    pub zoom: f32,
    pub cam_info: [[f32; 4]; 4],
}

pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vec3::<f32>::new(0., 0., 0.),
            up: Vec3::<f32>::new(0., 1., 0.),
            focus: Vec3::<f32>::new(0., 0., 0.),
            far: 10000000.,
            near: 0.0001,
            zoom: 1.,
            view_angle: 35. * PI / 180.,
            cam_info: [[0.; 4]; 4],
            //[cam_pos[0], cam_pos[1], cam_pos[2], view_port_center[0]]
            //[view_port_center[1], view_port_center[2], pixel_width, pixel_height]
            //[half_width, half_height, x[0], x[1]]
            //[x[2], y[0], y[1], y[2]]
        }
    }
}

impl Camera {
    pub fn new(
        position: Option<Vec3<f32>>,
        up: Option<Vec3<f32>>,
        focus: Option<Vec3<f32>>,
        near: Option<f32>,
        far: Option<f32>,
        view_angle: Option<f32>,
        zoom: Option<f32>,
    ) -> Self {
        Self {
            position: position.unwrap_or(Vec3::new(0.0, 0.0, 0.0)),
            up: up.unwrap_or(Vec3::new(0., 1., 0.0)),
            focus: focus.unwrap_or(Vec3::new(0.0, 0.0, 1.0)),
            near: near.unwrap_or(0.1),
            far: far.unwrap_or(10000000000.),
            view_angle: view_angle.unwrap_or(30.),
            zoom: zoom.unwrap_or(1.),
            cam_info: [[0.; 4]; 4],
        }
    }

    pub fn movement(&mut self, direction: Direction, rotate: &bool) {
        let factor = (&self.focus - &self.position).length() as f32 / 200.;

        let movement_direction = match direction {
            Direction::Backward => (&self.focus - &self.position).normalize() * -factor,
            Direction::Forward => (&self.focus - &self.position).normalize() * factor,
            Direction::Down => self.up.clone().normalize() * -factor,
            Direction::Up => self.up.clone().normalize() * factor,
            Direction::Left => (&self.focus - &self.position).cross(&self.up).normalize() * -factor,
            Direction::Right => (&self.focus - &self.position).cross(&self.up).normalize() * factor,
        };
        let new_position = &self.position + &movement_direction;
        if !rotate {
            self.focus += movement_direction;
        } else {
            match direction {
                Direction::Up | Direction::Down => {
                    self.up = (&self.focus - &new_position)
                        .cross(&(self.up).cross(&(&self.focus - &self.position)).normalize())
                        .normalize();
                }
                _ => {}
            }
        }
        self.position = new_position;
    }

    pub fn update_cam_info(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        let aspect_ratio = size.width as f32 / size.height as f32;
        self.cam_info[0][0] = self.position.v[0];
        self.cam_info[0][1] = self.position.v[1];
        self.cam_info[0][2] = self.position.v[2];
        self.cam_info[2][1] = (self.view_angle / (2. * self.zoom)).tan() * self.near;
        self.cam_info[2][0] = aspect_ratio * self.cam_info[2][1];
        let z = (&self.focus - &self.position).normalize();
        let x = self.up.cross(&z).normalize();
        self.cam_info[2][2] = x.v[0];
        self.cam_info[2][3] = x.v[1];
        self.cam_info[3][0] = x.v[2];
        let y = z.cross(&x).normalize();
        self.cam_info[3][1] = y.v[0];
        self.cam_info[3][2] = y.v[1];
        self.cam_info[3][3] = y.v[2];

        let view_port_center = &self.position + z * self.near;
        self.cam_info[0][3] = view_port_center.v[0];
        self.cam_info[1][0] = view_port_center.v[1];
        self.cam_info[1][1] = view_port_center.v[2];
    }
}

pub struct CamManager {
    pub camera: Camera,
    pub camera_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl CamManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mut camera: Camera,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        camera.update_cam_info(size);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[camera.cam_info]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let mut manager = Self {
            camera_buffer,
            bind_group_layout,
            bind_group,
            camera,
        };
        manager.update_buffers(queue);
        manager
    }

    pub fn update_buffers(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera.cam_info]),
        );
    }
}
