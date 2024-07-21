use std::sync::Arc;
use std::sync::Mutex;

use rayon::max_num_threads;
use rayon::prelude::*;
use crate::ray::Ray;
use crate::Vec3;
use crate::mesh::{Mesh, Triangle};

pub struct Node<'a>{
    pub bounds: [f32; 6],
    pub start_triangle: usize,
    pub triangle_count: u128,
    pub left_node: Option<&'a Node<'a>>,
    pub right_node: Option<&'a Node<'a>>,
}

pub struct BVH<'a> {
    pub nodes: Vec<Node<'a>>,
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
        triangle_count: 0,
        left_node: None,
        right_node: None,
    };
    let bvh = BVH{
        nodes: vec![root_node],
        triangles,
    };
    bvh
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