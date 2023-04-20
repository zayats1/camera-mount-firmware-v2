use super::stepper::*;
use embedded_hal::digital::v2::OutputPin;

struct StepperWithDriver<T: OutputPin> {
    steps: usize,
    dir_pin: T,
    clk: T,
}

impl<T: OutputPin> Stepper for StepperWithDriver<T> {
    fn rotate(&mut self, dir: super::stepper::Directions) {
        self.set_dir(dir);
    }

    fn steps(&mut self, steps: usize, dir: super::stepper::Directions) {
        self.set_dir(dir);
        for _ in 0..self.steps {
            self.clk.set_high().unwrap_or_default();
            // delay
            self.clk.set_low().unwrap_or_default();
            // delay
        }
    }

    fn stop(&mut self) {
        todo!()
    }

    fn hold(&mut self) {
        todo!()
    }
}

impl<T: OutputPin> StepperWithDriver<T> {
    fn set_dir(&mut self, dir: super::stepper::Directions) {
        match dir {
            Directions::Forward => self.dir_pin.set_high().unwrap_or_default(),
            Directions::Backward => self.dir_pin.set_low().unwrap_or_default(),
        }
    }
}
