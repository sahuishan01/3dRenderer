use std::f32::consts::PI;

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


#[derive(Clone)]
pub struct  Camera {
    pub position: Vec3<f32>,
    pub up: Vec3<f32>,
    pub focus: Vec3<f32>,
    pub near: f32,
    pub far: f32,
    pub view_angle: f32,
}

pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down
}

impl Default for Camera {
    fn default() -> Self {
        Camera{
            position: Vec3::<f32>::new(0., 0., 0.),
            up: Vec3::<f32>::new(0., 1., 0.),
            focus: Vec3::<f32>::new(0., 0., 0.),
            far: 10000000.,
            near: 0.0001,
            view_angle: 35.* PI/180.,
        }
    }
    
}

impl Camera{
    pub fn new(position: Option<Vec3<f32>>, up: Option<Vec3<f32>>, focus: Option<Vec3<f32>>, near: Option<f32>, far: Option<f32>, view_angle: Option<f32>) -> Self{
        Self {
            position: position.unwrap_or(Vec3::new(0.0, 0.0, 0.0)),
            up: up.unwrap_or(Vec3::new(0., 1., 0.0)),
            focus: focus.unwrap_or(Vec3::new(0.0, 0.0, 1.0)),
            near: near.unwrap_or(0.1),
            far: far.unwrap_or(10000000000.),
            view_angle: view_angle.unwrap_or(30.)
        }
    }
    
    pub fn movement(&mut self, direction: Direction, rotate: &bool){


        let movement_direction = match direction{
            Direction::Backward => (&self.focus - &self.position).normalize() * -0.1,
            Direction::Forward => (&self.focus - &self.position).normalize() * 0.1,
            Direction::Down => self.up.clone().normalize() * -0.1,
            Direction::Up => self.up.clone().normalize() * 0.1,
            Direction::Left => (&self.focus - &self.position).cross(&self.up).normalize() * -0.1,
            Direction::Right => (&self.focus - &self.position).cross(&self.up).normalize() * 0.1,
        };
        let new_position = &self.position + &movement_direction;
        if !rotate {
            self.focus += movement_direction;
        }
        else{
            match direction {
                Direction::Up | Direction::Down => {
                    self.up = (&self.focus - &new_position).cross(&(&self.up).cross(&(&self.focus - &self.position)).normalize()).normalize();
                },
                _=>{}
            }
        }
        self.position = new_position;
    }
}
