use core::fmt;
use std::{ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign}};


// conversion to f64
pub trait ToF64 {
    fn to_f64(self) -> f64;
}

impl ToF64 for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToF64 for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

impl ToF64 for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToF64 for u32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}



// main struct

#[derive(Debug, Clone, Copy)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}


// Addition traits

    // scalar addition
impl<T> AddAssign<T> for Vec3<T> where T: AddAssign + Copy{
    fn add_assign(&mut self, rhs: T) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

impl<T> Add<T> for Vec3<T>
where 
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: T) -> Self::Output {
        Vec3::<T> {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs
        }
    }
}

    // addition with self
impl<T> AddAssign<&Vec3<T>> for Vec3<T>                 // Modifying self
where
    T: AddAssign + Copy,
{
    fn add_assign(&mut self, rhs: &Vec3<T>) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<T> Add<&Vec3<T>> for &Vec3<T>                      // Returning new Vec3 without moving data
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: &Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}


impl<T> Add<&Vec3<T>> for Vec3<T>                       // Returning new Vec3 without moving first element
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: &Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> Add for Vec3<T>                                // Returning new Vec3 with both inputs moved
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}


// subraction traits
    // subtraction with scalar
impl<T> SubAssign<T> for Vec3<T> where T: SubAssign + Copy{
    fn sub_assign(&mut self, rhs: T) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}
    // subraction with self
impl<T> SubAssign<&Vec3<T>> for Vec3<T>                 // Modifying self
where
    T: SubAssign + Copy,
{
    fn sub_assign(&mut self, rhs: &Vec3<T>){
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<T> Sub<&Vec3<T>> for &Vec3<T>                      // Returning new Vec3 without moving data
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn sub(self, rhs: &Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> Sub<&Vec3<T>> for Vec3<T>                       // Returning new Vec3 without moving first element
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn sub(self, rhs: &Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> Sub for Vec3<T>                                // Returning new Vec3 with both inputs moved
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn sub(self, rhs: Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}


// multiplication traits
    // multiplication with scalar
impl<T> MulAssign<T> for Vec3<T>
where 
    T: MulAssign + Copy
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

    // multiplication with self
impl<T> MulAssign<&Vec3<T>> for Vec3<T>                   // Modifying self
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: &Vec3<T>) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl<T> Mul<&Vec3<T>> for &Vec3<T>                      // Returning new Vec3 without moving data
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: &Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<T> Mul<&Vec3<T>> for Vec3<T>                       // Returning new Vec3 without moving first element
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: &Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<T> Mul for Vec3<T>                                // Returning new Vec3 with both inputs moved
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<T> Mul<T> for Vec3<T>                                // Multiplying with scalar
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: T) -> Vec3<T> {
        Vec3::<T> {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}


// divison traits
    // division by scalar
impl<T> DivAssign<T> for Vec3<T>
where 
    T: DivAssign + Copy
{
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}
impl<T> Div<T> for Vec3<T>
where 
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: T) -> Self::Output {
        Vec3::<T> {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs
        }
    }
}

    // division with self
impl<T> DivAssign<&Vec3<T>> for Vec3<T>                     // Modifying self
where
    T: DivAssign + Copy,
{
    fn div_assign(&mut self, rhs: &Vec3<T>) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}


impl<T> Div<&Vec3<T>> for &Vec3<T>                      // Returning new Vec3 without moving data
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: &Vec3<T>) -> Self::Output {
        Vec3::<T> {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl<T> Div<&Vec3<T>> for Vec3<T>                       // Returning new Vec3 without moving first element
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: &Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl<T> Div for Vec3<T>                                // Returning new Vec3 with both inputs moved
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: Vec3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}


// other required implementations
impl<T> Vec3<T>
where T: Copy + Mul<Output = T> + Add<Output = T> + ToF64 + Sub<Output = T>
{

    pub fn new(x: T, y: T, z: T) -> Self {
        Vec3 { x, y, z }
    }

    pub fn squared_length(&self) -> T {
       self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f64 {
        self.squared_length().to_f64().sqrt()
    }

    pub fn dot(self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: &Self) -> Vec3<T> {
        Vec3::<T> {
            x: self.y * other.z - other.y * self.z,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.y,
        }
    }

    pub fn unit_vector(self) -> Vec3<f64> {
        let length = &self.length();
        Vec3::<f64>{
            x: &self.x.to_f64() / length,
            y: &self.y.to_f64() / length,
            z: &self.z.to_f64() / length,
        }
    }
    
}

// implementing display for writing into file
impl<T: fmt::Display> fmt::Display for Vec3<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

pub fn write_color<T: ToF64 + Copy>(out: &mut [u8], pixel_color: &Vec3<T>){
    let r = pixel_color.x;
    let g = pixel_color.y;
    let b = pixel_color.z;

    // Translate the [0,1] component values to the byte range [0,255].
    let r = (255.999 * r.to_f64()).round() as u8;
    let g = (255.999 * g.to_f64()).round() as u8;
    let b = (255.999 * b.to_f64()).round() as u8;

    out[0] = r;
    out[1] = g;
    out[2] = b;
    out[3] = 255;

}
