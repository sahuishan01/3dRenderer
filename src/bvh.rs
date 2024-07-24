use std::sync::Arc;
use std::sync::Mutex;

use rayon::max_num_threads;
use rayon::prelude::*;
use crate::ray::Ray;
use crate::utils::MinHeap;
use crate::Vec3;
use crate::mesh::{Mesh, Triangle};

#[derive(Clone)]
pub struct Node{
    pub bounds: [f32; 6],
    pub start_triangle: usize,
    pub triangle_count: usize,
    pub left_node: Option<usize>,
    pub right_node: Option<usize>,
}

#[derive(Clone)]
pub struct BVH {
    pub nodes: Vec<Node>,
    pub triangles: Vec<Triangle>
}

pub fn create_bvh(mesh: &Mesh) -> BVH{
    let mut triangles = vec![Triangle::new(); mesh.faces.len()];
    let mut triangle_centers: Vec<Vec3<f32>> = vec![Vec3::<f32>::e_new(); mesh.faces.len()];
    let chunk_size = mesh.faces.len().div_ceil(max_num_threads());
    let bounds = Arc::new(Mutex::new([f32::MAX, f32::MAX, f32::MAX, f32::MIN, f32::MIN, f32::MIN]));
    triangle_centers.par_chunks_mut(chunk_size).zip(triangles.par_chunks_mut(chunk_size)).enumerate().for_each(|(chunk_idx, (chunk1, chunk2))|{
        let mut local_min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut local_max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for i in 0..chunk1.len(){
            let global_index = chunk_idx * chunk_size + i;
            if global_index < mesh.faces.len() {
                let face = mesh.faces[global_index];
                let v1 = face.v1 as usize;
                let v2 = face.v2 as usize;
                let v3 = face.v3 as usize;
                chunk1[i].x = (mesh.vertices[v1].x + mesh.vertices[v2].x + mesh.vertices[v3].x) / 3.0;
                chunk1[i].y = (mesh.vertices[v1].y + mesh.vertices[v2].y + mesh.vertices[v3].y) / 3.0;
                chunk1[i].z = (mesh.vertices[v1].z + mesh.vertices[v2].z + mesh.vertices[v3].z) / 3.0;
                chunk2[i].vertices[0] = mesh.vertices[v1];
                chunk2[i].vertices[1] = mesh.vertices[v2];
                chunk2[i].vertices[2] = mesh.vertices[v3];
                chunk2[i].normal = mesh.normals[global_index];
                let x_max = chunk2[i].vertices[0].x.max(chunk2[i].vertices[1].x.max(chunk2[i].vertices[2].x));
                let y_max = chunk2[i].vertices[0].y.max(chunk2[i].vertices[1].y.max(chunk2[i].vertices[2].y));
                let z_max = chunk2[i].vertices[0].z.max(chunk2[i].vertices[1].z.max(chunk2[i].vertices[2].z));
                let x_min = chunk2[i].vertices[0].x.min(chunk2[i].vertices[1].x.min(chunk2[i].vertices[2].x));
                let y_min = chunk2[i].vertices[0].y.min(chunk2[i].vertices[1].y.min(chunk2[i].vertices[2].y));
                let z_min = chunk2[i].vertices[0].z.min(chunk2[i].vertices[1].z.min(chunk2[i].vertices[2].z));

                local_min.x = local_min.x.min(x_min);
                local_min.y = local_min.y.min(y_min);
                local_min.z = local_min.z.min(z_min);
                local_max.x = local_max.x.max(x_max);
                local_max.y = local_max.y.max(y_max);
                local_max.z = local_max.z.max(z_max);
            }
            else{
                break
            }
        };
        let mut bound = bounds.lock().unwrap();
        bound[0] = bound[0].min(local_min.x);
        bound[1] = bound[1].min(local_min.y);
        bound[2] = bound[2].min(local_min.z);
        bound[3] = bound[3].max(local_max.x);
        bound[4] = bound[4].max(local_max.y);
        bound[5] = bound[5].max(local_max.z);
    });
    
    let final_bounds = bounds.lock().unwrap();
    println!("Min bound: ({}, {}, {})", final_bounds[0], final_bounds[1], final_bounds[2]);
    println!("Max bound: ({}, {}, {})", final_bounds[3], final_bounds[4], final_bounds[5]);
    let root_node = Node{
        bounds: *final_bounds,
        start_triangle: 0,
        triangle_count: triangle_centers.len(),
        left_node: None,
        right_node: None,
    };
    let mut depth = 10_u8;
    let mut nodes = Vec::from([root_node]);
    depth -= 1;
    create_nodes(&mut triangle_centers, &mut triangles, &mut nodes, &mut depth, 0);
    let bvh = BVH{
        nodes,
        triangles,
    };
    bvh
}

pub fn create_nodes(triangle_centers: &mut Vec<Vec3<f32>>, triangles: &mut Vec<Triangle>, nodes: &mut Vec<Node>, depth: &mut u8, parent_node_index: usize){
    let bounds = nodes[parent_node_index].bounds;
    let x_size = bounds[3] - bounds[0];
    let y_size = bounds[4] - bounds[1];
    let z_size = bounds[5] - bounds[2];
    if *depth == 0_u8 { return };
    
    if x_size > y_size.max(z_size) {
        let split_point = (bounds[3] + bounds[0]) / 2.;
        let left_bounds = [bounds[0], bounds[1], bounds[2], split_point, bounds[4], bounds[5]];
        let right_bounds = [split_point, bounds[1], bounds[2], bounds[3], bounds[4], bounds[5]];
        let mut right_indices: MinHeap<usize> = MinHeap::new();
        let mut last_left_triangle_index = nodes[parent_node_index].start_triangle;
        for i in nodes[parent_node_index].start_triangle..nodes[parent_node_index].triangle_count{
            if triangle_centers[i].x <= split_point{
                if !right_indices.is_empty(){
                    let swap_index = right_indices.pop().unwrap();
                    triangles.swap(i, swap_index);
                    triangle_centers.swap(i, swap_index);
                }
                last_left_triangle_index = i;
            }
            else{
                right_indices.push(i);
            }
        }
        let left_node = Node {
            bounds: left_bounds,
            start_triangle: nodes[parent_node_index].start_triangle,
            triangle_count: last_left_triangle_index - nodes[parent_node_index].start_triangle,
            left_node: None,
            right_node: None,
        };
        nodes.push(left_node);
        let left_node_index = nodes.len() - 1;
        nodes[parent_node_index].left_node = Some(left_node_index);
        let mut current_dept = *depth - 1;
        create_nodes(triangle_centers, triangles, nodes, &mut current_dept, left_node_index);
        if !(nodes[left_node_index].triangle_count == nodes[parent_node_index].triangle_count){
            let right_node = Node {
                bounds: right_bounds,
                start_triangle: last_left_triangle_index,
                triangle_count: nodes[parent_node_index].triangle_count - nodes[left_node_index].triangle_count,
                left_node: None,
                right_node: None,
            };
            nodes.push(right_node);
            let right_node_index = nodes.len() - 1;
            nodes[parent_node_index].right_node = Some(right_node_index);
            create_nodes(triangle_centers, triangles, nodes, &mut current_dept, right_node_index);
        }
    } else if y_size > z_size {
        let split_point = (bounds[4] + bounds[1]) / 2.;
        let left_bounds = [bounds[0], bounds[1], bounds[2], bounds[3], split_point, bounds[5]];
        let right_bounds = [bounds[0], split_point, bounds[2], bounds[3], bounds[4], bounds[5]];
        let mut right_indices: MinHeap<usize> = MinHeap::new();
        let mut last_left_triangle_index = nodes[parent_node_index].start_triangle;
        for i in nodes[parent_node_index].start_triangle..nodes[parent_node_index].triangle_count{
            if triangle_centers[i].y <= split_point{
                if !right_indices.is_empty(){
                    let swap_index = right_indices.pop().unwrap();
                    triangles.swap(i, swap_index);
                    triangle_centers.swap(i, swap_index);
                }
                last_left_triangle_index = i;
            }
            else{
                right_indices.push(i);
            }
        }
        let left_node = Node {
            bounds: left_bounds,
            start_triangle: nodes[parent_node_index].start_triangle,
            triangle_count: last_left_triangle_index - nodes[parent_node_index].start_triangle,
            left_node: None,
            right_node: None,
        };
        nodes.push(left_node);
        let left_node_index = nodes.len() - 1;
        nodes[parent_node_index].left_node = Some(left_node_index);
        let mut current_dept = *depth - 1;
        create_nodes(triangle_centers, triangles, nodes, &mut current_dept, left_node_index);
        if !(nodes[left_node_index].triangle_count == nodes[parent_node_index].triangle_count){
            let right_node = Node {
                bounds: right_bounds,
                start_triangle: last_left_triangle_index,
                triangle_count: nodes[parent_node_index].triangle_count - nodes[left_node_index].triangle_count,
                left_node: None,
                right_node: None,
            };
            nodes.push(right_node);
            let right_node_index = nodes.len() - 1;
            nodes[parent_node_index].right_node = Some(right_node_index);
            create_nodes(triangle_centers, triangles, nodes, &mut current_dept, right_node_index);
        }
    } else {
        let split_point = (bounds[5] + bounds[2]) / 2.;
        let left_bounds = [bounds[0], bounds[1], bounds[2], bounds[3], bounds[4], split_point];
        let right_bounds = [bounds[0], bounds[1], split_point, bounds[3], bounds[4], bounds[5]];
        let mut right_indices: MinHeap<usize> = MinHeap::new();
        let mut last_left_triangle_index = nodes[parent_node_index].start_triangle;
        for i in nodes[parent_node_index].start_triangle..nodes[parent_node_index].triangle_count{
            if triangle_centers[i].z <= split_point{
                if !right_indices.is_empty(){
                    let swap_index = right_indices.pop().unwrap();
                    triangles.swap(i, swap_index);
                    triangle_centers.swap(i, swap_index);
                }
                last_left_triangle_index = i;
            }
            else{
                right_indices.push(i);
            }
        }
        let left_node = Node {
            bounds: left_bounds,
            start_triangle: nodes[parent_node_index].start_triangle,
            triangle_count: last_left_triangle_index - nodes[parent_node_index].start_triangle,
            left_node: None,
            right_node: None,
        };
        nodes.push(left_node);
        let left_node_index = nodes.len() - 1;
        nodes[parent_node_index].left_node = Some(left_node_index);
        let mut current_dept = *depth - 1;
        create_nodes(triangle_centers, triangles, nodes, &mut current_dept, left_node_index);
        if !(nodes[left_node_index].triangle_count == nodes[parent_node_index].triangle_count){
            let right_node = Node {
                bounds: right_bounds,
                start_triangle: last_left_triangle_index,
                triangle_count: nodes[parent_node_index].triangle_count - nodes[left_node_index].triangle_count,
                left_node: None,
                right_node: None,
            };
            nodes.push(right_node);
            let right_node_index = nodes.len() - 1;
            nodes[parent_node_index].right_node = Some(right_node_index);
            create_nodes(triangle_centers, triangles, nodes, &mut current_dept, right_node_index);
        }
    };
    println!("depth: {depth}");
}

pub fn intersect_aabb(ray: &Ray, aabb: &[f32; 6]) -> Option<f64> {
    let mut tmin = f64::NEG_INFINITY;
    let mut tmax = f64::INFINITY;

    let directions = [ray.direction.x, ray.direction.y, ray.direction.z];
    let origins = [ray.origin.x, ray.origin.y, ray.origin.z];
    let aabb = [aabb[0] as f64, aabb[1] as f64, aabb[2] as f64, aabb[3] as f64, aabb[4] as f64, aabb[5] as f64];

    for i in 0..3 {
        if directions[i].abs() > f64::EPSILON {
            let t1 = (aabb[i] - origins[i]) / directions[i];
            let t2 = (aabb[i + 3] - origins[i]) / directions[i];

            tmin = tmin.max(t1.min(t2));
            tmax = tmax.min(t1.max(t2));
        } else if origins[i] < aabb[i] || origins[i] > aabb[i + 3] {
            // Ray is parallel to slab, but origin is not within slab
            return None;
        }
    }

    if tmax >= tmin && tmax >= 0.0 {
        Some(tmin.max(0.0))  // Return 0.0 if tmin is negative (ray starts inside AABB)
    } else {
        None
    }
}

pub fn is_on_box_boundary(ray: &Ray, bounds: &[f32; 6],) -> bool {
    let inv_dir = Vec3 {
        x: 1.0 / ray.direction.x,
        y: 1.0 / ray.direction.y,
        z: 1.0 / ray.direction.z,
    };

    let min_bounds = Vec3::new(bounds[0] as f64, bounds[1] as f64,bounds[2] as f64);
    let max_bounds = Vec3::new(bounds[3] as f64, bounds[4] as f64,bounds[5] as f64);

    let t1 = (min_bounds.x - ray.origin.x) * inv_dir.x;
    let t2 = (max_bounds.x - ray.origin.x) * inv_dir.x;
    let t3 = (min_bounds.y - ray.origin.y) * inv_dir.y;
    let t4 = (max_bounds.y - ray.origin.y) * inv_dir.y;
    let t5 = (min_bounds.z - ray.origin.z) * inv_dir.z;
    let t6 = (max_bounds.z - ray.origin.z) * inv_dir.z;

    let txmin = t1.min(t2);
    let txmax = t1.max(t2);
    let tymin = t3.min(t4);
    let tymax = t3.max(t4);
    let tzmin = t5.min(t6);
    let tzmax = t5.max(t6);

    let tmin = txmin.max(tymin).max(tzmin);
    let tmax = txmax.min(tymax).min(tzmax);

    if tmax < tmin || tmax <= 0.0 {
        return false; // Ray doesn't intersect the box at all
    }

    let intersection_point = ray.at(tmin);

    let dist = (intersection_point - ray.origin).length() * 3.0_f64.sqrt();

    let mut min_subs = intersection_point - min_bounds;
    min_subs.x = min_subs.x.abs();
    min_subs.y = min_subs.y.abs();
    min_subs.z = min_subs.z.abs();

    let mut max_subs = intersection_point - max_bounds;
    max_subs.x = max_subs.x.abs();
    max_subs.y = max_subs.y.abs();
    max_subs.z = max_subs.z.abs();

    let x_thresh = ((max_bounds.x - min_bounds.x) * dist / 500.).min(0.052);
    let y_thresh = ((max_bounds.y - min_bounds.y) * dist / 500.).min(0.052);
    let z_thresh = ((max_bounds.z - min_bounds.z) * dist / 500.).min(0.052);

    // Check if the intersection point is on an edge
    let on_x_edge = (min_subs.y < x_thresh  || max_subs.y < x_thresh ) && (min_subs.z < x_thresh  || max_subs.z < x_thresh );
    let on_y_edge = (min_subs.x < y_thresh  || max_subs.x < y_thresh ) && (min_subs.z < y_thresh  || max_subs.z < y_thresh );
    let on_z_edge = (min_subs.y < z_thresh  || max_subs.y < z_thresh ) && (min_subs.x < z_thresh  || max_subs.x < z_thresh );
    
    on_x_edge || on_y_edge || on_z_edge
}