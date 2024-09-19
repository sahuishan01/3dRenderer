pub mod vector;

use std::mem;
use rand::Rng;

pub fn struct_to_bytes<T>(data: &T) -> &[u8] {
    unsafe {
        // Get a pointer to the data and cast it to a pointer to bytes
        let ptr = data as *const T as *const u8;
        // Create a slice of bytes from the pointer
        std::slice::from_raw_parts(ptr, mem::size_of::<T>())
    }
}

pub fn generate_random_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EntityCount{
    pub count: u32,
} 
