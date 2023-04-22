#![allow(unused)]

#[derive(Default)]
pub enum Directions {
    #[default]
    Forward,
    Backward,
}

pub trait Stepper {
    fn steps<F: FnMut(u32) -> ()>(&mut self, steps: usize, delay: F);
    fn stop(&mut self);
    fn hold(&mut self);
    fn set_dir(&mut self, dir: Directions);
    fn set_speed(&mut self, speed: u32);
    fn set_acceleration(&mut self, acceleration: i32);
}
