use crate::vector::{ToF64, Vec3};

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3<f64>,
    pub direction: Vec3<f64>,
}

impl Ray{
    pub fn new<T: ToF64>(origin: Vec3<T>, direction: Vec3<T>) -> Self {
        Ray {
            origin: Vec3::new(origin.x.to_f64(), origin.y.to_f64(), origin.z.to_f64()),
            direction: Vec3::new( direction.x.to_f64(), direction.y.to_f64(), direction.z.to_f64())
        }
    }
    
    pub fn at<Ray: Copy>(&self, t: f64) -> Vec3<f64> {
        self.origin + self.direction * t
    }
}


