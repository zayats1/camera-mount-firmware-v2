pub enum Directions {
    Forward,
    Backward,
}

pub trait Stepper {
    fn rotate(&mut self, dir: Directions);
    fn steps(&mut self, steps: usize, dir: Directions);
    fn stop(&mut self);
    fn hold(&mut self);
}
