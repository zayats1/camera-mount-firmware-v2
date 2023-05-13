/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(unused)]

use embedded_hal::digital::v2::OutputPin;

#[derive(Default)]
pub enum Direction {
    #[default]
    Forward,
    Backward,
    Stop,
}

pub struct StepperWithDriver<T: OutputPin, U: OutputPin> {
    dir_pin: T,
    clk: U,
    dir: Direction,
    speed: u32,
    acceleration: i32,
}

impl<T, U> StepperWithDriver<T, U>
where
    T: OutputPin,
    U: OutputPin,
{
    pub fn set_dir(&mut self, dir: Direction) {
        self.dir = dir
    }

    pub fn steps<F: FnMut(u32)>(&mut self, mut delay_ms: F) {
        match self.dir {
            Direction::Forward => {
                self.dir_pin.set_high().unwrap_or_default();
            }
            Direction::Backward => {
                self.dir_pin.set_low().unwrap_or_default();
            }
            Direction::Stop => {
                return;
            }
        }
        let delay_time = 1000 / self.speed;
        // It is unstopable for now
        self.clk.set_high().unwrap_or_default();
        delay_ms(delay_time); // for prototype
        self.clk.set_low().unwrap_or_default();
        delay_ms(delay_time);
    }

    fn stop(&mut self) {
        todo!()
    }

    pub fn set_speed(&mut self, speed: u32) {
        self.speed = speed;
    }

    fn set_acceleration(&mut self, acceleration: i32) {
        self.acceleration = acceleration;
    }

    pub fn new(dir_pin: T, clk: U, speed: u32, acceleration: i32) -> Self {
        Self {
            dir_pin,
            clk,
            dir: Direction::default(),
            speed,
            acceleration,
        }
    }
}
