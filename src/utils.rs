use rayon::prelude::*;
use crate::ray::Ray;
use crate::vector::{Vec3, write_color};

pub fn hit_sphere(center: Vec3<f64>, radius: f64, r: &Ray) -> f64 {
    let oc = center - &r.origin;
    let a = r.direction.dot(&r.direction);
    let b = -2.0 * r.direction.dot(&oc);
    let c = oc.dot(&oc) - radius * radius;
    b * b - 4. * a * c
}

pub fn ray_color(r: Ray) -> Vec3<f64> {
    let t = hit_sphere(Vec3::new(0., 0., 1.), 0.5, &r);
    if t > 0.0 {
        let n = (r.at::<Ray>(t) - Vec3::new(0., 0., -1.)).unit_vector();
        return Vec3::new(n.x + 1., n.y + 1., n.z + 1.) * 0.5;
    }
    let unit_direction = r.direction.unit_vector();
    let a = (unit_direction.y + 1.) * 0.5;
    Vec3::new(0.5, 0.7, 1.0) * a + Vec3::new(1.0, 1.0, 1.0) * (1. - a)
}

pub fn generate_gradient(width: u32, img: &mut [u8], pixel00_loc: Vec3<f64>, pixel_delta_u: Vec3<f64>, pixel_delta_v: Vec3<f64>, camera_center: Vec3<f64>) {
    img.par_chunks_mut(width as usize * 4 ).enumerate().for_each( |(i, row)| {
        for j in 0..width {
            let pixel_center =
                pixel00_loc + pixel_delta_u * j as f64 + pixel_delta_v * i as f64;
            let ray_direction = pixel_center - camera_center;
            let r: Ray = Ray::new(camera_center, ray_direction);
            let pixel_color = ray_color(r);
            let offset = j as usize * 4;
            write_color(&mut row[offset..offset + 4], &pixel_color);
        }
    });
}
