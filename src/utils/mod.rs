pub mod vector;
pub mod mesh;
pub mod bvh;

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

 use std::collections::BinaryHeap;
 use std::cmp::Ordering;


 // Wrapper struct for our elements
 #[derive(PartialEq, Eq)]
 struct MinHeapElement<T: Ord>(T);
 // Implement Ord and PartialOrd to reverse the ordering
 impl<T: Ord> Ord for MinHeapElement<T> {
     fn cmp(&self, other: &Self) -> Ordering {
         other.0.cmp(&self.0)
     }
 }
 impl<T: Ord> PartialOrd for MinHeapElement<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
         Some(self.cmp(other))
     }
 }
 // MinHeap struct
 pub struct MinHeap<T: Ord> {
     heap: BinaryHeap<MinHeapElement<T>>,
 }

 impl<T: Ord> MinHeap<T> {
     pub fn new() -> Self {
         MinHeap { heap: BinaryHeap::new() }
     }
     pub fn push(&mut self, item: T) {
         self.heap.push(MinHeapElement(item));
     }
     pub fn pop(&mut self) -> Option<T> {
         self.heap.pop().map(|MinHeapElement(item)| item)
     }

     pub fn peek(&self) -> Option<&T> {
         self.heap.peek().map(|MinHeapElement(item)| item)
     }
     pub fn len(&self) -> usize {
         self.heap.len()
     }

     pub fn is_empty(&self) -> bool {
         self.heap.is_empty()
     }
 }


