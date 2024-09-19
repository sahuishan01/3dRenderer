use crate::vector::Vec3;

#[derive(Clone)]
pub struct Ray<'a> {
    pub origin: &'a Vec3<f32>,
    pub direction: Vec3<f32>,
    pub inv: Vec3<f32>,
}


impl<'a> Ray<'a>{
    pub fn new(origin: &'a Vec3<f32>, direction: Vec3<f32>, inv: Vec3<f32>) -> Self{
        Ray{
            origin,
            direction,
            inv
        }
    }
    
    pub fn at(&self, t: f32) -> Vec3<f32> {
        self.origin + &self.direction * t
    }
}

