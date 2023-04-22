use super::stepper::*;
use embedded_hal::digital::v2::OutputPin;

pub struct StepperWithDriver<T: OutputPin, U: OutputPin> {
    dir_pin: T,
    clk: U,
    dir: Directions,
    speed: u32,
    acceleration: i32,
}

impl<T, U> Stepper for StepperWithDriver<T, U>
where
    T: OutputPin,
    U: OutputPin,
{
    fn set_dir(&mut self, dir: super::stepper::Directions) {
        self.dir = dir
    }

    fn steps<F: FnMut(u32) -> ()>(&mut self, steps: usize, mut delay_ms: F) {
        match self.dir {
            Directions::Forward => {
                self.dir_pin.set_high().unwrap_or_default();
            }
            Directions::Backward => {
                self.dir_pin.set_low().unwrap_or_default();
            }
        }
        let delay_time = 1000 / self.speed;
        for _ in 0..steps {
            // It is unstopable for now
            self.clk.set_high().unwrap_or_default();
            delay_ms(delay_time); // for prototype
            self.clk.set_low().unwrap_or_default();
            delay_ms(delay_time);
        }
    }

    fn stop(&mut self) {
        todo!()
    }

    fn hold(&mut self) {
        todo!()
    }

    fn set_speed(&mut self, speed: u32) {
        self.speed = speed;
    }

    fn set_acceleration(&mut self, acceleration: i32) {
        self.acceleration = acceleration;
    }
}

impl<T, U: OutputPin> StepperWithDriver<T, U>
where
    T: OutputPin,
    U: OutputPin,
{
    pub fn new(dir_pin: T, clk: U, speed: u32, acceleration: i32) -> Self {
        Self {
            dir_pin,
            clk,
            dir: Directions::default(),
            speed,
            acceleration,
        }
    }
}
