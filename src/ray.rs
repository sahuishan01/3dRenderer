
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
    
    pub fn at(&self, t: f64) -> Vec3<f64> {
        self.origin + self.direction * t
    }
}


#[derive(Clone, Copy)]
pub struct  Camera {
    pub position: Vec3<f64>,
    pub up: Vec3<f64>,
    pub focus: Vec3<f64>,
    pub near: f64,
    pub far: f64,
    pub view_angle: f64,
}

pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down
}

impl Camera{
    pub fn new(position: Option<Vec3<f64>>, up: Option<Vec3<f64>>, focus: Option<Vec3<f64>>, near: Option<f64>, far: Option<f64>, view_angle: Option<f64>) -> Self{
        Self {
            position: position.unwrap_or(Vec3::new(0.0, 0.0, 0.0)),
            up: up.unwrap_or(Vec3::new(0., 1., 0.0)),
            focus: focus.unwrap_or(Vec3::new(0.0, 0.0, 1.0)),
            near: near.unwrap_or(0.1),
            far: far.unwrap_or(10000000000.),
            view_angle: view_angle.unwrap_or(30.)
        }
    }
    
    pub fn movement(&mut self, direction: Direction){
        let movement_direction = match direction{
            Direction::Backward => (self.focus - self.position).normalize() * -0.1,
            Direction::Forward => (self.focus - self.position).normalize() * 0.1,
            Direction::Down => self.up.normalize() * -0.1,
            Direction::Up => self.up.normalize() * 0.1,
            Direction::Left => (self.focus - self.position).cross(&self.up).normalize() * -0.1,
            Direction::Right => (self.focus - self.position).cross(&self.up).normalize() * 0.1,
        };
        self.position += &movement_direction;
        self.focus += &movement_direction;
    }

    pub fn rotate(&mut self, dx: f64, dy: f64, sensitivity: Option<f64>){
        let sensitivity = sensitivity.unwrap_or(0.002);
        let view_direction = (self.focus - self.position).normalize();
        let right = view_direction.cross(&self.up);
        let true_up =   right.cross(&view_direction);
        let horizontal_rotation = Quaternion::from_axis_angle(&true_up, -dx * sensitivity);
        let vertical_rotation = Quaternion::from_axis_angle(&right, -dy * sensitivity);
        let combined_rotation = horizontal_rotation.multiply(&vertical_rotation);
        let new_view_dir = combined_rotation.rotate_vector(&view_direction);
    
        // Calculate new position
        let distance = self.focus - self.position.length();
        self.position = self.focus - (&new_view_dir * &distance);
    
        // Calculate new up vector
        self.up = combined_rotation.rotate_vector(&self.up);
    }
}


#[derive(Clone, Copy)]
pub struct Quaternion {
    w: f64,
    x: f64,
    y: f64,
    z: f64,
}

impl Quaternion {
    fn from_axis_angle(axis: &Vec3<f64>, angle: f64) -> Quaternion {
        let sin_half = (angle / 2.0).sin();
        Quaternion {
            w: (angle / 2.0).cos(),
            x: axis.x * sin_half,
            y: axis.y * sin_half,
            z: axis.z * sin_half,
        }
    }

    fn multiply(&self, other: &Quaternion) -> Quaternion {
        Quaternion {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }

    fn rotate_vector(&self, v: &Vec3<f64>) -> Vec3<f64> {
        let q = Quaternion { w: 0.0, x: v.x, y: v.y, z: v.z };
        let q_conjugate = Quaternion { w: self.w, x: -self.x, y: -self.y, z: -self.z };
        let rotated = self.multiply(&q).multiply(&q_conjugate);
        Vec3 { x: rotated.x, y: rotated.y, z: rotated.z }
    }
}
