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

fn rotate_around_axis(v: &Vec3<f32>, axis: &Vec3<f32>, angle: f32) -> Vec3<f32> {
    // Rodrigues' rotation formula: v*cos + (axis×v)*sin + axis*(axis·v)*(1-cos)
    let cos = angle.cos();
    let sin = angle.sin();
    let dot = axis.dot(v);
    let cross = axis.cross(v);
    Vec3::<f32> {
        v: [
            v.v[0] * cos + cross.v[0] * sin + axis.v[0] * dot * (1.0 - cos),
            v.v[1] * cos + cross.v[1] * sin + axis.v[1] * dot * (1.0 - cos),
            v.v[2] * cos + cross.v[2] * sin + axis.v[2] * dot * (1.0 - cos),
        ],
    }
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

    pub fn movement(&mut self, direction: Direction, rotate: &bool, amount: f32) {
        if *rotate {
            // Orbit camera: rotate position around the focus point.
            let offset = &self.position - &self.focus;
            let distance = offset.length() as f32;
            // Scale sensitivity inversely with distance so a 1-pixel drag
            // produces the same apparent motion regardless of zoom.
            let sensitivity = (0.09 / distance).clamp(0.0001, 0.01);
            let angle = amount * sensitivity;

            // Derive orbit axes from a fixed world-up so the yaw axis never drifts.
            let world_up = Vec3::new(0.0f32, 1.0f32, 0.0f32);
            let forward = (&self.focus - &self.position).normalize();
            let dot = world_up.dot(&forward) as f32;
            let proj = &forward * dot;
            let mut yaw_axis = &world_up - &proj;
            if yaw_axis.length() < 0.001 {
                // Gimbal lock: fallback to Z-up reference
                let fallback = Vec3::new(0.0f32, 0.0f32, 1.0f32);
                let dot2 = fallback.dot(&forward) as f32;
                let proj2 = &forward * dot2;
                yaw_axis = &fallback - &proj2;
            }
            yaw_axis = yaw_axis.normalize();
            let right = yaw_axis.cross(&forward).normalize();

            let new_offset = match direction {
                Direction::Left | Direction::Right => {
                    let sign = if matches!(direction, Direction::Left) { 1.0 } else { -1.0 };
                    rotate_around_axis(&offset, &yaw_axis, sign * angle)
                }
                Direction::Up | Direction::Down => {
                    let sign = if matches!(direction, Direction::Up) { -1.0 } else { 1.0 };
                    let new_off = rotate_around_axis(&offset, &right, sign * angle);
                    // Clamp pitch to avoid looking straight up/down (gimbal lock)
                    let new_forward = (&self.focus - &(&self.focus + &new_off)).normalize();
                    if new_forward.dot(&world_up).abs() < 0.99 {
                        new_off
                    } else {
                        offset
                    }
                }
                _ => offset,
            };
            self.position = &self.focus + &new_offset;
            // Lock up to the derived yaw axis so it can never drift
            self.up = yaw_axis;
        } else {
            // Pan camera: translate both position and focus
            let factor = (&self.focus - &self.position).length() as f32 / 1000. * amount;
            let world_up = Vec3::new(0.0f32, 1.0f32, 0.0f32);
            let forward = (&self.focus - &self.position).normalize();
            let dot = world_up.dot(&forward) as f32;
            let proj = &forward * dot;
            let mut pan_up = &world_up - &proj;
            if pan_up.length() < 0.001 {
                let fallback = Vec3::new(0.0f32, 0.0f32, 1.0f32);
                let dot2 = fallback.dot(&forward) as f32;
                let proj2 = &forward * dot2;
                pan_up = &fallback - &proj2;
            }
            pan_up = pan_up.normalize();
            let pan_right = pan_up.cross(&forward).normalize();

            let movement_direction = match direction {
                Direction::Backward => forward * -factor,
                Direction::Forward => forward * factor,
                Direction::Down => pan_up * -factor,
                Direction::Up => pan_up * factor,
                Direction::Left => pan_right * -factor,
                Direction::Right => pan_right * factor,
            };
            self.position = &self.position + &movement_direction;
            self.focus = &self.focus + &movement_direction;
        }
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
