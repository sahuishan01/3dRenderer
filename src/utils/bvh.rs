use std::sync::Arc;
use std::sync::Mutex;

use rayon::max_num_threads;
use rayon::prelude::*;
use crate::utils::MinHeap;
use crate::utils::mesh::Mesh;

use super::EntityCount;


#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::NoUninit)]
pub struct Triangle2{
    pub normal: [f32; 3],
    pub p1_: f32,
    pub v1: [f32; 3], 
    pub p2_: f32,
    pub v2: [f32; 3],
    pub p3_: f32,
    pub v3: [f32; 3],
    pub p4_: f32,
}
impl Default for Triangle2 {
    fn default() -> Self {
       Self { 
            normal: [0., 0., 0.],
            v1: [0., 0., 0.],
            v2: [0., 0., 0.],
            v3: [0., 0., 0.], 
            p1_ : 0.,
            p2_ : 0.,
            p3_ : 0.,
            p4_ : 0.,
       }
}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::NoUninit)]
pub struct Node{
    pub bounds: [f32; 6],
    pub start_triangle: usize,
    pub triangle_count: usize,
    pub left_node: usize,
    pub right_node: usize,
    pub padding_: [u32; 6], 
}

#[repr(C)]
pub struct BVH {
    pub nodes: Vec<Node>,
    pub triangles: Vec<Triangle2>
}

pub fn create_bvh(mesh: &Mesh, depth: u8) -> BVH{
    let mut triangles: std::vec::Vec<Triangle2> = Vec::with_capacity(mesh.faces.len()); 
    let mut triangle_centers: Vec<[f32; 3]> = vec![[0., 0., 0.]; mesh.faces.len()];
    let chunk_size = mesh.faces.len().div_ceil(max_num_threads());
    let bounds = Arc::new(Mutex::new([f32::MAX, f32::MAX, f32::MAX, f32::MIN, f32::MIN, f32::MIN]));
    triangle_centers.par_chunks_mut(chunk_size).zip(triangles.par_chunks_mut(chunk_size)).enumerate().for_each(|(chunk_idx, (chunk1, chunk2))|{
        let mut local_min = [f32::MAX, f32::MAX, f32::MAX];
        let mut local_max = [f32::MIN, f32::MIN, f32::MIN];
        for i in 0..chunk1.len(){
            let global_index = chunk_idx * chunk_size + i;
            if global_index < mesh.faces.len() {
                let face = mesh.faces[global_index];
                let v1 = face[0];
                let v2 = face[1];
                let v3 = face[2];
                chunk1[i][0] = (mesh.vertices[v1][0] + mesh.vertices[v2][0] + mesh.vertices[v3][0]) / 3.0;
                chunk1[i][1] = (mesh.vertices[v1][1] + mesh.vertices[v2][1] + mesh.vertices[v3][1]) / 3.0;
                chunk1[i][2] = (mesh.vertices[v1][2] + mesh.vertices[v2][2] + mesh.vertices[v3][2]) / 3.0;
                chunk2[i].v1 = [mesh.vertices[v1][0], mesh.vertices[v1][1], mesh.vertices[v1][2]];
                chunk2[i].v2 = [mesh.vertices[v2][0], mesh.vertices[v2][1], mesh.vertices[v2][2]];
                chunk2[i].v3 = [mesh.vertices[v3][0], mesh.vertices[v3][1], mesh.vertices[v3][2]];
                chunk2[i].normal = [mesh.normals[global_index][0], mesh.normals[global_index][1], mesh.normals[global_index][2]];
                let x_max = chunk2[i].v1[0].max(chunk2[i].v2[0].max(chunk2[i].v3[0]));
                let y_max = chunk2[i].v1[1].max(chunk2[i].v2[1].max(chunk2[i].v3[1]));
                let z_max = chunk2[i].v1[2].max(chunk2[i].v2[2].max(chunk2[i].v3[2]));
                let x_min = chunk2[i].v1[0].min(chunk2[i].v2[0].min(chunk2[i].v3[0]));
                let y_min = chunk2[i].v1[1].min(chunk2[i].v2[1].min(chunk2[i].v3[1]));
                let z_min = chunk2[i].v1[2].min(chunk2[i].v2[2].min(chunk2[i].v3[2]));

                local_min[0] = local_min[0].min(x_min);
                local_min[1] = local_min[1].min(y_min);
                local_min[2] = local_min[2].min(z_min);
                local_max[0] = local_max[0].max(x_max);
                local_max[1] = local_max[1].max(y_max);
                local_max[2] = local_max[2].max(z_max);
            }
            else{
                break
            }
        };
        let mut bound = bounds.lock().unwrap();
        bound[0] = bound[0].min(local_min[0]);
        bound[1] = bound[1].min(local_min[1]);
        bound[2] = bound[2].min(local_min[2]);
        bound[3] = bound[3].max(local_max[0]);
        bound[4] = bound[4].max(local_max[1]);
        bound[5] = bound[5].max(local_max[2]);
    });
    let final_bounds = bounds.lock().unwrap();
    let root_node = Node{
        bounds: *final_bounds,
        start_triangle: 0,
        triangle_count: triangle_centers.len(),
        left_node: 0,
        right_node: 0,
        padding_: [0, 0, 0, 0, 0, 0]
    };
    let mut nodes = vec![root_node];
    let depth = depth - 1;
    create_nodes(&mut triangle_centers, &mut triangles, &mut nodes, depth, 0);
    BVH{
        nodes,
        triangles,
    }
}

pub fn create_nodes(triangle_centers: &mut Vec<[f32; 3]>, triangles: &mut Vec<Triangle2>, nodes: &mut Vec<Node>, depth: u8, parent_node_index: usize){
    let bounds = nodes[parent_node_index].bounds;
    let x_size = bounds[3] - bounds[0];
    let y_size = bounds[4] - bounds[1];
    let z_size = bounds[5] - bounds[2];
    if depth == 0 { return };
    let compare_index = if x_size > y_size.max(z_size){
        0
    }
    else if y_size > z_size{
        1
    }
    else{
        2
    };
    
    let split_point = (bounds[compare_index + 3] + bounds[compare_index]) / 2.;
    let mut left_bounds = bounds;
    let mut right_bounds = bounds;
    left_bounds[compare_index + 3] = split_point;
    right_bounds[compare_index] = split_point;
    let mut right_indices = MinHeap::new();
    let mut last_left_triangle_index = nodes[parent_node_index].start_triangle;
    let parent_last_index = nodes[parent_node_index].triangle_count + nodes[parent_node_index].start_triangle;
    for i in nodes[parent_node_index].start_triangle..parent_last_index{
        if triangle_centers[i][compare_index] <= split_point {
            if !right_indices.is_empty(){
                let swap_index = right_indices.pop().unwrap();
                triangles.swap(i, swap_index);
                triangle_centers.swap(i, swap_index);
                last_left_triangle_index = swap_index;
            }
            else{
                last_left_triangle_index = i;
            }
        }
        else{
            right_indices.push(i);
        }
    }
    let current_dept = depth - 1;
    if last_left_triangle_index > nodes[parent_node_index].start_triangle{
        let left_node = Node {
            bounds: left_bounds,
            start_triangle: nodes[parent_node_index].start_triangle,
            triangle_count: last_left_triangle_index - nodes[parent_node_index].start_triangle,
            left_node: 0,
            right_node: 0,
            padding_: [0, 0, 0, 0, 0, 0]
        };
        nodes.push(left_node);
        let left_node_index = nodes.len() - 1;
        nodes[parent_node_index].left_node = left_node_index;
        create_nodes(triangle_centers, triangles, nodes, current_dept, left_node_index);
    }
    if parent_last_index > last_left_triangle_index{
        let right_node = Node{
            bounds: right_bounds,
            start_triangle: last_left_triangle_index,
            triangle_count: parent_last_index - last_left_triangle_index,
            left_node: 0,
            right_node: 0,
            padding_: [0, 0, 0, 0, 0, 0]
        };
        nodes.push(right_node);
        let right_node_index = nodes.len() - 1;
        nodes[parent_node_index].right_node = right_node_index;
        create_nodes(triangle_centers, triangles, nodes, current_dept, right_node_index);
    }
}

pub struct BvhManager {
    pub nodes_buffer: wgpu::Buffer,
    pub triangles_buffer: wgpu::Buffer,
    pub nodes_count_buffer: wgpu::Buffer,
    pub triangles_count_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bvh: BVH,
}

impl BvhManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, mesh: &Mesh) -> Self {
        let (nodes_buffer, triangles_buffer, nodes_count_buffer, triangles_count_buffer, bind_group, bind_group_layout, bvh) = Self::create_buffers_and_bind_group(device, mesh);


        let mut manager = Self {
            nodes_buffer,
            triangles_buffer,
            nodes_count_buffer,
            triangles_count_buffer,
            bind_group,
            bind_group_layout,
            bvh,
        };

        manager.update_buffers(queue);
        manager

    }

    fn create_buffers_and_bind_group(device: &wgpu::Device, mesh: &Mesh) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::Buffer, wgpu::BindGroup, wgpu::BindGroupLayout, BVH){
            
        let bvh = create_bvh(mesh, 25);
        let nodes_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Nodes buffer"),
            size: (std::mem::size_of::<Node>() * bvh.nodes.len()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let triangles_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Triangles buffer"),
            size: (std::mem::size_of::<Triangle2>() * bvh.triangles.len()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let nodes_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Nodes count buffer"),
            size: std::mem::size_of::<EntityCount>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let triangles_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Triangles count buffer"),
            size: std::mem::size_of::<EntityCount>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
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
                        ty: wgpu::BufferBindingType::Storage { read_only: false},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Bvh Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: nodes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: triangles_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: nodes_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: triangles_count_buffer.as_entire_binding(),
                },
            ],
            label: Some("Bvh Bind Group"),
        });

        (nodes_buffer, triangles_buffer, nodes_count_buffer, triangles_count_buffer, bind_group, bind_group_layout, bvh)
    }

    fn update_buffers(&mut self, queue: &wgpu::Queue){
        queue.write_buffer(&self.nodes_buffer, 0, bytemuck::cast_slice(&self.bvh.nodes));
        let _count = EntityCount{count: self.bvh.nodes.len() as u32};
    }
}

//pub fn intersect_aabb(ray: &Ray, bounding_box: &[f32; 6]) -> Option<f32> {
//    let tx1 = (bounding_box[0] - ray.origin.v[0]) * ray.inv.v[0];
//    let tx2 = (bounding_box[3] - ray.origin.v[0]) * ray.inv.v[0];
//    let t1 = tx1.min(tx2);
//    let t2 = tx1.max(tx2);
//
//    let ty1 = (bounding_box[1] - ray.origin.v[1]) * ray.inv.v[1];
//    let ty2 = (bounding_box[4] - ray.origin.v[1]) * ray.inv.v[1];
//    let t3 = ty1.min(ty2);
//    let t4 = ty1.max(ty2);
//
//    let tz1 = (bounding_box[2] - ray.origin.v[2]) * ray.inv.v[2];
//    let tz2 = (bounding_box[5] - ray.origin.v[2]) * ray.inv.v[2];
//    let t5 = tz1.min(tz2);
//    let t6 = tz1.max(tz2);
//
//    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
//    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
//
//    if tmax >= tmin && tmax > 0.0 {
//        Some(tmin as f32)
//    } else {
//        None
//    }
//}
//
//pub fn moller_trumbore_intersection (origin: &Vec3<f32>, direction: &Vec3<f32>, triangle: &Triangle2) -> bool {
//	let e1 = &triangle.vertices[1] - &triangle.vertices[0];
//	let e2 = &triangle.vertices[2] - &triangle.vertices[0];
//
//	let ray_cross_e2 = direction.cross(&e2);
//	let det = e1.dot(&ray_cross_e2);
//
//	if det > -f32::EPSILON && det < f32::EPSILON {
//		return false; // This ray is parallel to this triangle.
//	}
//
//	let inv_det = 1.0 / det;
//    let v1 = &triangle.vertices[0];
//	let s = origin - v1;
//	let u = inv_det * s.dot(&ray_cross_e2);
//	if u < 0.0 || u > 1.0 {
//		return false;
//	}
//
//	let s_cross_e1 = s.cross(&e1);
//	let v = inv_det * direction.dot(&s_cross_e1);
//	if v < 0.0 || u + v > 1.0 {
//		return false;
//	}
//	// At this stage we can compute t to find out where the intersection point is on the line.
//	let t = inv_det * e2.dot(&s_cross_e1);
//
//	if t > f32::EPSILON { // ray intersection
//		let intersection_point = origin + direction * t;
//		return true;
//	}
//	else { // This means that there is a line intersection but not a ray intersection.
//		return false;
//	}
//}
//
//pub fn is_on_box_boundary(ray: &Ray, bounds: &[f32; 6],) -> bool {
//    let inv_dir = Vec3 {
//        v: [1.0 / ray.direction.v[0], 1.0 / ray.direction.v[1], 1.0 / ray.direction.v[2]]
//    };
//
//    let min_bounds = Vec3::new(bounds[0], bounds[1],bounds[2]);
//    let max_bounds = Vec3::new(bounds[3], bounds[4],bounds[5]);
//
//    let t1 = (min_bounds.v[0] - ray.origin.v[0]) * inv_dir.v[0];
//    let t2 = (max_bounds.v[0] - ray.origin.v[0]) * inv_dir.v[0];
//    let t3 = (min_bounds.v[1] - ray.origin.v[1]) * inv_dir.v[1];
//    let t4 = (max_bounds.v[1] - ray.origin.v[1]) * inv_dir.v[1];
//    let t5 = (min_bounds.v[2] - ray.origin.v[2]) * inv_dir.v[2];
//    let t6 = (max_bounds.v[2] - ray.origin.v[2]) * inv_dir.v[2];
//
//    let txmin = t1.min(t2);
//    let txmax = t1.max(t2);
//    let tymin = t3.min(t4);
//    let tymax = t3.max(t4);
//    let tzmin = t5.min(t6);
//    let tzmax = t5.max(t6);
//
//    let tmin = txmin.max(tymin).max(tzmin);
//    let tmax = txmax.min(tymax).min(tzmax);
//
//    if tmax < tmin || tmax <= 0.0 {
//        return false; // Ray doesn't intersect the box at all
//    }
//
//    let intersection_point = ray.at(tmin);
//
//    let dist = ((&intersection_point - ray.origin).length() * 3.0_f64.sqrt()) as f32;
//
//    let mut min_subs = &intersection_point - &min_bounds;
//    min_subs.v[0] = min_subs.v[0].abs();
//    min_subs.v[1] = min_subs.v[1].abs();
//    min_subs.v[2] = min_subs.v[2].abs();
//
//    let mut max_subs = &intersection_point - &max_bounds;
//    max_subs.v[0] = max_subs.v[0].abs();
//    max_subs.v[1] = max_subs.v[1].abs();
//    max_subs.v[2] = max_subs.v[2].abs();
//
//    let x_thresh = ((max_bounds.v[0] - min_bounds.v[0]) * dist / 500.).min(0.052);
//    let y_thresh = ((max_bounds.v[1] - min_bounds.v[1]) * dist / 500.).min(0.052);
//    let z_thresh = ((max_bounds.v[2] - min_bounds.v[2]) * dist / 500.).min(0.052);
//
//    // Check if the intersection point is on an edge
//    let on_x_edge = (min_subs.v[1] < x_thresh  || max_subs.v[1] < x_thresh ) && (min_subs.v[2] < x_thresh  || max_subs.v[2] < x_thresh );
//    let on_y_edge = (min_subs.v[0] < y_thresh  || max_subs.v[0] < y_thresh ) && (min_subs.v[2] < y_thresh  || max_subs.v[2] < y_thresh );
//    let on_z_edge = (min_subs.v[1] < z_thresh  || max_subs.v[1] < z_thresh ) && (min_subs.v[0] < z_thresh  || max_subs.v[0] < z_thresh );
//
//    on_x_edge || on_y_edge || on_z_edge
//}

#[cfg(test)]
mod test{
    use std::{path::PathBuf, str::FromStr, time::Instant};

    use crate::utils::mesh::load_mesh;

    use super::create_bvh;


    #[test]
    fn bvh_test(){

        let mesh = load_mesh(&PathBuf::from_str("../../STLS/Raw/1.stl").unwrap()).unwrap();
        let start = Instant::now();
        let _bvh = create_bvh(&mesh, 20);
        println!("BVH time: {:?} milliseconds", start.elapsed().as_millis());
    }
}
