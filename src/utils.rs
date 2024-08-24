// use rayon::prelude::*;
// use crate::bvh::{intersect_aabb, is_on_box_boundary, moller_trumbore_intersection, Node};
// use crate::ray::Ray;
// use crate::vector::{Vec3, write_color};
// use crate::Triangle2;

use std::mem;
use bytemuck::NoUninit;
use rand::Rng;

pub fn struct_to_bytes<T>(data: &T) -> &[u8] {
    unsafe {
        // Get a pointer to the data and cast it to a pointer to bytes
        let ptr = data as *const T as *const u8;
        // Create a slice of bytes from the pointer
        std::slice::from_raw_parts(ptr, mem::size_of::<T>())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, NoUninit)]
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

pub fn generate_random_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}
// pub fn hitSphere(r: &Ray, sphere_radius: f32) -> f32 {
//     let a = r.direction.dot(&r.direction);
//     let b = 2.0 * r.origin.dot(&r.direction);
//     let c = r.origin.dot(&r.origin) - sphere_radius * sphere_radius;
//     let discriminant = b * b - 4.0 * a * c;

//     if discriminant < 0.0 {
//         return 0.; // No intersection
//     }

//     let sqrt_discriminant = discriminant.sqrt();
//     let tmin = (-b - sqrt_discriminant) / (2.0 * a);
//     let tmax = (-b + sqrt_discriminant) / (2.0 * a);

//     if tmin > 0.0 {
//         tmin
//     } else if tmax > 0.0 {
//         tmax
//     } else {
//         0.
//     }
// }



// pub fn generate_gradient(width: u32, img: &mut [u8], pixel00_loc: Vec3<f32>, pixel_delta_u: Vec3<f32>, pixel_delta_v: Vec3<f32>, camera_center: Vec3<f32>) {
//     img.par_chunks_mut(width as usize * 4 ).enumerate().for_each( |(i, row)| {
//         for j in 0..width {
//             let pixel_center =
//                 pixel00_loc + pixel_delta_u * j as f32 + pixel_delta_v * i as f32;
//             let ray_direction = pixel_center - camera_center;
//             let r: Ray = Ray::new(camera_center, ray_direction);
//             let pixel_color = ray_color(r);
//             let offset = j as usize * 4;
//             write_color(&mut row[offset..offset + 4], pixel_color);
//         }
//     });
// }
// use std::collections::BinaryHeap;
// use std::cmp::Ordering;

// // Wrapper struct for our elements
// #[derive(PartialEq, Eq)]
// struct MinHeapElement<T: Ord>(T);

// // Implement Ord and PartialOrd to reverse the ordering
// impl<T: Ord> Ord for MinHeapElement<T> {
//     fn cmp(&self, other: &Self) -> Ordering {
//         other.0.cmp(&self.0)
//     }
// }

// impl<T: Ord> PartialOrd for MinHeapElement<T> {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// // MinHeap struct
// pub struct MinHeap<T: Ord> {
//     heap: BinaryHeap<MinHeapElement<T>>,
// }

// impl<T: Ord> MinHeap<T> {
//     pub fn new() -> Self {
//         MinHeap { heap: BinaryHeap::new() }
//     }

//     pub fn push(&mut self, item: T) {
//         self.heap.push(MinHeapElement(item));
//     }

//     pub fn pop(&mut self) -> Option<T> {
//         self.heap.pop().map(|MinHeapElement(item)| item)
//     }

//     pub fn peek(&self) -> Option<&T> {
//         self.heap.peek().map(|MinHeapElement(item)| item)
//     }

//     pub fn len(&self) -> usize {
//         self.heap.len()
//     }

//     pub fn is_empty(&self) -> bool {
//         self.heap.is_empty()
//     }
// }


// pub fn generate_image(
//     width: u32,
//     img: &mut [u8],
//     view_port_center: Vec3<f32>,
//     half_width: f32,
//     half_height: f32,
//     pixel_width: f32,
//     pixel_height: f32,
//     camera_center: Vec3<f32>,
//     nodes: &Vec<Node>,
//     triangles: &Vec<Triangle2>,
//     x: Vec3<f32>,
//     y: Vec3<f32>,
// ) {    
//     let sub_offsets = [
//             (0.5, 0.5),

//             // (0.25, 0.25),
//             // (0.75, 0.25),
//             // (0.25, 0.75),
//             // (0.75, 0.75),

//             // (0.25, 0.25),
//             // (0.5, 0.25),
//             // (0.75, 0.25),
//             // (0.25, 0.5),
//             // (0.5, 0.5),
//             // (0.75, 0.5),
//             // (0.25, 0.75),
//             // (0.5, 0.75),
//             // (0.75, 0.75),
//         ];
//     img.par_chunks_mut(width as usize * 4 ).enumerate().for_each( |(i, row)| {
//         for j in 0..width {
//             let mut hit = false;
//             let mut t_hit = false;
//             let mut hit_count = 0.;
//             let mut t_normal = Vec3{v: [0., 0., 0.]};
//             for (sub_u, sub_v) in &sub_offsets {
                
//                 let u = (j as f32 + sub_u) * pixel_width - half_width;
//                 let v = (i as f32 + sub_v) * pixel_height - half_height;
//                 let pixel_center = &view_port_center + &x * u + &y * v;
//                 if i==half_height as usize && j==half_width as u32{
//                     println!("{:?}", &pixel_center)
//                 }
//                 let ray_direction = (pixel_center - &camera_center).normalize();
//                 let ray_inv = Vec3::new(1. / ray_direction.v[0], 1. / ray_direction.v[1], 1. / ray_direction.v[2]).normalize();
//                 let ray: Ray = Ray::new(&camera_center, ray_direction, ray_inv);
//                 let hit_distance = hitSphere(&ray, 5.);
//                 if hit_distance > 0.{
//                     hit = true;
//                     let hit_point = ray.at(hit_distance);
//                     t_normal = (hit_point - t_normal).normalize();
//                 }
                
//                 // let mut hit_boxes = MinHeap::new();
//                 // if intersect_aabb(&ray, &nodes[0].bounds).is_none() {continue;}
//                 // hit_boxes.push(0);
//                 // let mut indices = vec![0];
//                 // while let Some(index) = indices.pop() {
//                 //     let node = &nodes[index];
//                 //     let left_hit_dist = match node.left_node{
//                 //         Some(left_node) => intersect_aabb(&ray, &nodes[left_node].bounds),
//                 //         None => None
//                 //     };
//                 //     let right_hit_dist = match node.right_node{
//                 //         Some(right_node) => intersect_aabb(&ray, &nodes[right_node].bounds),
//                 //         None => None
//                 //     };
//                 //     match left_hit_dist {
//                 //         Some(dist) => {
//                 //             match right_hit_dist{
//                 //                 Some(right_dist) => {
//                 //                     if right_dist < dist {
//                 //                         indices.push(nodes[index].left_node.unwrap());
//                 //                         // hit_boxes.push(indices.last().unwrap().clone());
//                 //                         indices.push(nodes[index].right_node.unwrap());
//                 //                         // hit_boxes.push(indices.last().unwrap().clone());
//                 //                     }
//                 //                     else{
//                 //                         indices.push(nodes[index].right_node.unwrap());
//                 //                         // hit_boxes.push(indices.last().unwrap().clone());
//                 //                         indices.push(nodes[index].left_node.unwrap());
//                 //                         // hit_boxes.push(indices.last().unwrap().clone());
//                 //                     }
//                 //                 }
//                 //                 None => {
//                 //                     indices.push(nodes[index].left_node.unwrap());
//                 //                     // hit_boxes.push(indices.last().unwrap().clone());
//                 //                 }
//                 //             };
//                 //         },
//                 //         None => {
//                 //             match right_hit_dist{
//                 //                 Some(_) => {
//                 //                     indices.push(nodes[index].right_node.unwrap());
//                 //                     hit_boxes.push(indices.last().unwrap().clone());
//                 //                 },
//                 //                 None => {
//                 //                     // break;
//                 //                     // if nodes[index].triangle_count > 0 {
//                 //                     //     println!("{:?}", nodes[index].triangle_count);
//                 //                     // }
//                 //                     for i in nodes[index].start_triangle..nodes[index].start_triangle+nodes[index].triangle_count{
//                 //                         if moller_trumbore_intersection(&ray.origin, &ray.direction, &triangles[i]){
//                 //                             t_hit = true;
//                 //                             break;
//                 //                         }
//                 //                     }
//                 //                     if t_hit{break;}
//                 //                 }
//                 //             }
//                 //         }
//                 //     }
//                 // }
                
//                 // while let Some(value) = hit_boxes.pop(){
//                 //     if is_on_box_boundary(&ray, &nodes[value].bounds){
//                 //         hit = true;
//                 //         break;
//                 //     }
//                 // };
//             }
//             let mut red_shade = 0.;
//             if hit {
//                 red_shade = (3.14 - t_normal.angle(&Vec3{v:[1., 0., 0.0]})) / 3.14;
//                 if red_shade > 1. {
//                     println!("angle: {red_shade}")
//                 }
//             }
//             // if red_shade > 0. {
//             //     println!("{red_shade}")
//             // }
//             let pixel_color = [red_shade, 0. , 0. , 1.];
//             // let pixel_color =  if t_hit {[0., 0., 0., 1.]} else if hit {[1., 0., 0., 1.]} else {[1., 1., 1., 1.]};
//             let offset = j as usize * 4;
//             write_color(&mut row[offset..offset + 4], pixel_color);
//         }
//     });
// }


// pub fn generate_image2(
//     width: u32,
//     img: &mut [u8],
//     pixel00_loc: Vec3<f32>,
//     pixel_delta_u: Vec3<f32>,
//     pixel_delta_v: Vec3<f32>,
//     camera_center: &Vec3<f32>,
// ) {    
//     let sub_offsets = [
//             (0.5, 0.5),

//             // (0.25, 0.25),
//             // (0.75, 0.25),
//             // (0.25, 0.75),
//             // (0.75, 0.75),

//             // (0.25, 0.25),
//             // (0.5, 0.25),
//             // (0.75, 0.25),
//             // (0.25, 0.5),
//             // (0.5, 0.5),
//             // (0.75, 0.5),
//             // (0.25, 0.75),
//             // (0.5, 0.75),
//             // (0.75, 0.75),
//         ];
//     println!("pixel00 loc {:?}", &pixel00_loc);
//     img.par_chunks_mut(width as usize * 4 ).enumerate().for_each( |(i, row)| {
//         for j in 0..width {
//             let mut hit = false;
//             let mut t_normal = Vec3{v: [0., 0., 0.]};
//             for (sub_u, sub_v) in &sub_offsets {
//                 let pixel_center = &pixel00_loc + (&pixel_delta_u * (j as f32 + sub_u)) + (&pixel_delta_v * (i as f32 + sub_v));
//                 let ray_direction = (pixel_center - camera_center).normalize();
//                 if i==width as usize/2 && j==width/2{
//                     println!("pixel_direction {}", &ray_direction)
//                 }
//                 let ray_inv = Vec3::new(1. / ray_direction.v[0], 1. / ray_direction.v[1], 1. / ray_direction.v[2]).normalize();
//                 let ray: Ray = Ray::new(camera_center, ray_direction, ray_inv);
//                 let hit_distance = hitSphere(&ray, 5.);
//                 if hit_distance > 0.{
//                     hit = true;
//                     let hit_point = ray.at(hit_distance);
//                     t_normal = (hit_point - t_normal).normalize();
//                 }
//             }
//             let mut red_shade = 0.;
//             if hit {
//                 red_shade = (3.14 - t_normal.angle(&Vec3{v:[1., 0., 0.0]})) / 3.14;
//                 if red_shade > 1. {
//                     println!("angle: {red_shade}")
//                 }
//             }
//             let pixel_color = [red_shade, 0. , 0. , 1.];
//             let offset = j as usize * 4;
//             write_color(&mut row[offset..offset + 4], pixel_color);
//         }
//     });
// }

