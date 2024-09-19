use std::{fmt::Display, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign}};

use num_traits::{NumCast, ToPrimitive, Zero};

pub trait ConvertTo<U> {
    fn convert_to(self) -> U;
}

impl<T> ConvertTo<T> for T {
    fn convert_to(self) -> T {
        self
    }
}

fn convert_to<T, U>(value: T) -> U
where
    T: Into<U>,
{
    value.into()
}

#[derive(Debug, Clone)]
pub struct Vec3<T> {
    pub v: [T; 3],
}

impl<T: PartialEq> PartialEq for Vec3<T> {
    fn eq(&self, other: &Self) -> bool {
        self.v[0] == other.v[0] && self.v[1] == other.v[1] && self.v[2] == other.v[2]
    }
}

// Addition traits
// Vec += Number
impl<T: AddAssign + Copy> AddAssign<T> for Vec3<T>{
    fn add_assign(&mut self, rhs: T) {
        self.v[0] += rhs;
        self.v[1] += rhs;  
        self.v[2] += rhs;
    }
}
// Vec += &Vec
impl<T: AddAssign + Copy> AddAssign<&Vec3<T>> for Vec3<T> {
    fn add_assign(&mut self, rhs: &Vec3<T>) {
        self.v[0] += rhs.v[0];
        self.v[1] += rhs.v[1];
        self.v[2] += rhs.v[2];
    }
}
// Vec += Vec
impl<T: AddAssign + Copy> AddAssign<Vec3<T>> for Vec3<T> {
    fn add_assign(&mut self, rhs: Vec3<T>) {
        self.v[0] += rhs.v[0];
        self.v[1] += rhs.v[1];
        self.v[2] += rhs.v[2];
    }
}
// Vec + Number
impl<T> Add<T> for Vec3<T>
where 
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: T) -> Self::Output {
        Self::Output {
            v: [self.v[0] + rhs, self.v[1] + rhs, self.v[2] + rhs]
        }
    }
}
// Vec + Number
impl<T> Add<T> for &Vec3<T>
where 
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: T) -> Self::Output {
        Self::Output {
            v: [self.v[0] + rhs, self.v[1] + rhs, self.v[2] + rhs]
        }
    }
}
// &Vec + &Vec
impl<T> Add<&Vec3<T>> for &Vec3<T>
where   
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: &Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] + rhs.v[0], self.v[1] + rhs.v[1], self.v[2] + rhs.v[2]]
        }
    }
}
// Vec + &Vec
impl<T> Add<&Vec3<T>> for Vec3<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: &Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] + rhs.v[0], self.v[1] + rhs.v[1], self.v[2] + rhs.v[2]]
        }
    }
}
// &Vec + Vec
impl<T> Add<Vec3<T>> for &Vec3<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] + rhs.v[0], self.v[1] + rhs.v[1], self.v[2] + rhs.v[2]]
        }
    }
}
// Vec + Vec
impl<T> Add for Vec3<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn add(self, rhs: Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] + rhs.v[0], self.v[1] + rhs.v[1], self.v[2] + rhs.v[2]]
        }
    }
}


// subraction traits
// Vec -= Number
impl<T: SubAssign + Copy> SubAssign<T> for Vec3<T> {
    fn sub_assign(&mut self, rhs: T) {
        self.v[0] -= rhs;
        self.v[1] -= rhs;
        self.v[2] -= rhs;
    }
}
// Vec -= &Vec
impl<T: SubAssign + Copy> SubAssign<&Vec3<T>> for Vec3<T>
{
    fn sub_assign(&mut self, rhs: &Vec3<T>){
        self.v[0] -= rhs.v[0];
        self.v[1] -= rhs.v[1];
        self.v[2] -= rhs.v[2];
    }
}
// Vec -= Vec
impl<T: SubAssign + Copy> SubAssign<Vec3<T>> for Vec3<T>
{
    fn sub_assign(&mut self, rhs: Vec3<T>){
        self.v[0] -= rhs.v[0];
        self.v[1] -= rhs.v[1];
        self.v[2] -= rhs.v[2];
    }
}
// Vec - Number
impl<T> Sub<T> for Vec3<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn sub(self, rhs: T) -> Self::Output {
        Self::Output {
            v: [self.v[0] - rhs, self.v[1] - rhs, self.v[2] - rhs]
        }
    }
}
// &Vec - Number
impl<T> Sub<T> for &Vec3<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn sub(self, rhs: T) -> Self::Output {
        Self::Output {
            v: [self.v[0] - rhs, self.v[1] - rhs, self.v[2] - rhs]
        }
    }
}
// &Vec - &Vec
impl<T> Sub<&Vec3<T>> for &Vec3<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn sub(self, rhs: &Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] - rhs.v[0], self.v[1] - rhs.v[1], self.v[2] - rhs.v[2]]
        }
    }
}
// Vec - &Vec
impl<T> Sub<&Vec3<T>> for Vec3<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn sub(self, rhs: &Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] - rhs.v[0], self.v[1] - rhs.v[1], self.v[2] - rhs.v[2]]
        }
    }
}
// &Vec - Vec
impl<T> Sub<Vec3<T>> for &Vec3<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn sub(self, rhs: Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] - rhs.v[0], self.v[1] - rhs.v[1], self.v[2] - rhs.v[2]]
        }
    }
}
// Vec - Vec
impl<T> Sub for Vec3<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn sub(self, rhs: Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] - rhs.v[0], self.v[1] - rhs.v[1], self.v[2] - rhs.v[2]]
        }
    }
}


// multiplication traits
// Vec *= Number
impl<T: MulAssign + Copy> MulAssign<T> for Vec3<T>
{
    fn mul_assign(&mut self, rhs: T) {
        self.v[0] *= rhs;
        self.v[1] *= rhs;
        self.v[2] *= rhs;
    }
}
// Vec *= &Vec
impl<T: MulAssign + Copy> MulAssign<&Vec3<T>> for Vec3<T>
{
    fn mul_assign(&mut self, rhs: &Vec3<T>) {
        self.v[0] *= rhs.v[0];
        self.v[1] *= rhs.v[1];
        self.v[2] *= rhs.v[2];
    }
}
// Vec *= Vec
impl<T: MulAssign + Copy> MulAssign<Vec3<T>> for Vec3<T>
{
    fn mul_assign(&mut self, rhs: Vec3<T>) {
        self.v[0] *= rhs.v[0];
        self.v[1] *= rhs.v[1];
        self.v[2] *= rhs.v[2];
    }
}
// Vec * Number
impl<T> Mul<T> for Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            v: [self.v[0] * rhs, self.v[1] * rhs, self.v[2] * rhs]
        }
    }
}
// Vec * Number
impl<T> Mul<T> for &Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            v: [self.v[0] * rhs, self.v[1] * rhs, self.v[2] * rhs]
        }
    }
}
// &Vec * &Vec
impl<T> Mul<&Vec3<T>> for &Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: &Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] * rhs.v[0], self.v[1] * rhs.v[1], self.v[2] * rhs.v[2]]
        }
    }
}
// Vec * &Vec
impl<T> Mul<&Vec3<T>> for Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: &Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] * rhs.v[0], self.v[1] * rhs.v[1], self.v[2] * rhs.v[2]]
        }
    }
}
// &Vec * Vec
impl<T> Mul<Vec3<T>> for &Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] * rhs.v[0], self.v[1] * rhs.v[1], self.v[2] * rhs.v[2]]
        }
    }
}
// Vec * Vec
impl<T> Mul for Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn mul(self, rhs: Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] * rhs.v[0], self.v[1] * rhs.v[1], self.v[2] * rhs.v[2]]
        }
    }
}


// multiplication traits
// Vec /= Number
impl<T: DivAssign + Copy> DivAssign<T> for Vec3<T>
{
    fn div_assign(&mut self, rhs: T) {
        self.v[0] /= rhs;
        self.v[1] /= rhs;
        self.v[2] /= rhs;
    }
}
// Vec /= &Vec
impl<T: DivAssign + Copy> DivAssign<&Vec3<T>> for Vec3<T>
{
    fn div_assign(&mut self, rhs: &Vec3<T>) {
        self.v[0] /= rhs.v[0];
        self.v[1] /= rhs.v[1];
        self.v[2] /= rhs.v[2];
    }
}
// Vec /= Vec
impl<T: DivAssign + Copy> DivAssign<Vec3<T>> for Vec3<T>
{
    fn div_assign(&mut self, rhs: Vec3<T>) {
        self.v[0] /= rhs.v[0];
        self.v[1] /= rhs.v[1];
        self.v[2] /= rhs.v[2];
    }
}
// Vec / Number
impl<T> Div<T> for Vec3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            v: [self.v[0] / rhs, self.v[1] / rhs, self.v[2] / rhs]
        }
    }
}
// Vec / Number
impl<T> Div<T> for &Vec3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            v: [self.v[0] / rhs, self.v[1] / rhs, self.v[2] / rhs]
        }
    }
}
// &Vec / &Vec
impl<T> Div<&Vec3<T>> for &Vec3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: &Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] / rhs.v[0], self.v[1] / rhs.v[1], self.v[2] / rhs.v[2]]
        }
    }
}
// Vec / &Vec
impl<T> Div<&Vec3<T>> for Vec3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: &Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] / rhs.v[0], self.v[1] / rhs.v[1], self.v[2] / rhs.v[2]]
        }
    }
}
// &Vec / Vec
impl<T> Div<Vec3<T>> for &Vec3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] / rhs.v[0], self.v[1] / rhs.v[1], self.v[2] / rhs.v[2]]
        }
    }
}
// Vec / Vec
impl<T> Div for Vec3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec3<T>;
    fn div(self, rhs: Vec3<T>) -> Self::Output {
        Self::Output {
            v: [self.v[0] / rhs.v[0], self.v[1] / rhs.v[1], self.v[2] / rhs.v[2]]
        }
    }
}

// other required implementations
impl<T> Vec3<T>
where T:  PartialOrd + Copy + Mul<Output = T> + Copy + Add<Output = T> + Copy + Sub<Output = T> + Copy + ToPrimitive
{
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { v: [x, y, z]}
    }

    pub fn squared_length(&self) -> T {
       self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2]
    }

    pub fn length(&self) -> f64 {
        self.squared_length().to_f64().unwrap().sqrt()
    }

    pub fn dot(&self, other: &Self) -> T {
        self.v[0] * other.v[0] + self.v[1] * other.v[1] + self.v[2] * other.v[2]
    }

    pub fn cross(&self, other: &Self) -> Vec3<T> {
        Vec3::<T> {
            v: [self.v[1] * other.v[2] - other.v[1] * self.v[2], self.v[2] * other.v[0] - self.v[0] * other.v[2], self.v[0] * other.v[1] - self.v[1] * other.v[0]]
        }
    }

    pub fn normalize(&self) -> Vec3<f32> {
        let length = self.length() as f32;
        Vec3::<f32>{
            v: [&self.v[0].to_f32().unwrap() / length, &self.v[1].to_f32().unwrap() / length, &self.v[2].to_f32().unwrap() / length]
        }
    }
    
    pub fn max_component(&self) -> T {
        if self.v[0] > self.v[1] {
           if self.v[2] > self.v[0] {
                return self.v[2]
            }
            return self.v[0]
        }
        else{
            if self.v[2] > self.v[1] {
                return self.v[2]
            }
            return self.v[1]
        }
    }

    pub fn min_component(&self) -> T {
        if self.v[0] < self.v[1] {
            if self.v[2] < self.v[0] {
                 return self.v[2]
             }
             return self.v[0]
         }
         else{
             if self.v[2] < self.v[1] {
                 return self.v[2]
             }
             return self.v[1]
         }
    }

    pub fn to_array(&self) -> [T; 3]{
        [self.v[0], self.v[1], self.v[2]]
    }

    pub fn convert<U>(&self) -> Vec3<U>
    where
        U: Copy + NumCast + Zero,
    {
        let convert = |x: T| U::from(x).unwrap_or_else(U::zero);
        Vec3 {
            v: [convert(self.v[0]), convert(self.v[1]), convert(self.v[2])]
        }
    }
    
    pub fn angle(&self, other: &Self) -> f64 {
        (self.dot(other).to_f64().unwrap() / (self.length() * other.length())).acos()
    }
    
}

// implementing display for writing into file
impl<T: Display> Display for Vec3<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.v[0], self.v[1], self.v[2])
    }
}

pub fn write_color(out: &mut [u8], pixel_color: [f64; 4]){
    let r = pixel_color[0];
    let g = pixel_color[1];
    let b = pixel_color[2];
    let a = pixel_color[3];

    // Translate the [0,1] component values to the byte range [0,255].
    let r = (255.999 * r).round() as u8;
    let g = (255.999 * g).round() as u8;
    let b = (255.999 * b).round() as u8;
    let a = (255.999 * a).round() as u8;

    out[0] = r;
    out[1] = g;
    out[2] = b;
    out[3] = a;

}