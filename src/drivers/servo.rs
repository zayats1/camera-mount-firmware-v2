#![allow(unused)]

use embedded_hal::PwmPin;

pub struct Servo<'a, T: PwmPin<Duty = u16>> {
    pin: &'a mut T,
    duty_on_zero: u16,
    duty_on_90: u16,
    duty_per_degree: u16,
}
impl<'a, T: PwmPin<Duty = u16>> Servo<'a, T> {
    pub fn new(pin: &'a mut T, duty_on_zero: u16) -> Self {
        let duty_on_90 = duty_on_zero * 3;
        let duty_per_degree = (duty_on_90 - duty_on_zero) / 90;
        Self {
            pin,
            duty_on_zero,
            duty_on_90,
            duty_per_degree,
        }
    }

    pub fn set_angle(&mut self, angle: u16) {
        let duty_on_the_degree = self.duty_per_degree * angle + self.duty_on_zero;
        self.pin.set_duty(duty_on_the_degree);
    }

    pub fn to_center(self) {
        self.pin.set_duty(self.duty_on_90);
    }
}
