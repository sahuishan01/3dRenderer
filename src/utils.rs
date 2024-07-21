use rayon::prelude::*;
use crate::bvh::{is_on_box_boundary, BVH};
use crate::ray::Ray;
use crate::vector::{Vec3, write_color};

// pub fn ray_color(r: Ray) -> [f64; 4] {
//     let t = hit_sphere(Vec3::new(0., 0., 1.), 0.5, &r);
//     if t > 0.0 {
//         let n = (r.at(t) - Vec3::new(0., 0., -1.)).normalize();
//         return [(n.x + 1.) * 0.5, (n.y + 1.) * 0.5, (n.z + 1.) * 0.5, 1.0];
//     }
//     let unit_direction = r.direction.normalize();
//     let a = (unit_direction.y + 1.) * 0.5;
//     [(0.5 * a) + (1. * (1. -a)), (0.7 * a) + (1. * (1. -a)), (1. * a) + (1. * (1. -a)), 1.0]
// }


// pub fn generate_gradient(width: u32, img: &mut [u8], pixel00_loc: Vec3<f64>, pixel_delta_u: Vec3<f64>, pixel_delta_v: Vec3<f64>, camera_center: Vec3<f64>) {
//     img.par_chunks_mut(width as usize * 4 ).enumerate().for_each( |(i, row)| {
//         for j in 0..width {
//             let pixel_center =
//                 pixel00_loc + pixel_delta_u * j as f64 + pixel_delta_v * i as f64;
//             let ray_direction = pixel_center - camera_center;
//             let r: Ray = Ray::new(camera_center, ray_direction);
//             let pixel_color = ray_color(r);
//             let offset = j as usize * 4;
//             write_color(&mut row[offset..offset + 4], pixel_color);
//         }
//     });
// }


pub fn generate_image(
    width: u32,
    img: &mut [u8],
    view_port_center: Vec3<f64>,
    half_width: f64,
    half_height: f64,
    pixel_width: f64,
    pixel_height: f64,
    camera_center: Vec3<f64>,
    bvh: &BVH,
    X: Vec3<f64>,
    Y: Vec3<f64>,
) {    
    let sub_offsets = [
            // (0.5, 0.5),

            // (0.25, 0.25),
            // (0.75, 0.25),
            // (0.25, 0.75),
            // (0.75, 0.75),

            (0.25, 0.25),
            (0.5, 0.25),
            (0.75, 0.25),
            (0.25, 0.5),
            (0.5, 0.5),
            (0.75, 0.5),
            (0.25, 0.75),
            (0.5, 0.75),
            (0.75, 0.75),
        ];
    img.par_chunks_mut(width as usize * 4 ).enumerate().for_each( |(i, row)| {
        for j in 0..width {
            let mut hit = false;
            for (sub_u, sub_v) in &sub_offsets {
                let u = (j as f64 + sub_u) * pixel_width - half_width;
                let v = (i as f64 + sub_v) * pixel_height - half_height;
                let pixel_center = view_port_center + X * u + Y * v;
                let ray_direction = (pixel_center - camera_center).normalize();
                let ray: Ray = Ray::new(camera_center, ray_direction);
                for node in &bvh.nodes{
                    if is_on_box_boundary(&ray, &node.bounds) {
                        hit = true;
                    }
                }
            }
            let pixel_color =  if hit {[1., 0., 0., 1.]} else {[1., 1., 1., 1.]};
            let offset = j as usize * 4;
            write_color(&mut row[offset..offset + 4], pixel_color);
        }
    });
}
