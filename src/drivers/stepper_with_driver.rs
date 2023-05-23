/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use embedded_hal::digital::v2::OutputPin;

#[derive(Default, PartialEq)]
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
}

impl<T, U> StepperWithDriver<T, U>
where
    T: OutputPin,
    U: OutputPin,
{
    pub fn set_dir(&mut self, dir: Direction) {
        self.dir = dir
    }

    // TODO merge it to a better stepper motor library
    pub fn steps_with_timer(&mut self, timer_freq_micros: u32) {
        static mut IS_HIGH: bool = true;
        static mut CURRENT_TIME: u32 = 0;
        match self.dir {
            Direction::Forward => {
                self.dir_pin.set_high().unwrap_or_default();
            }
            Direction::Backward => {
                self.dir_pin.set_low().unwrap_or_default();
            }
            Direction::Stop => {
                self.stop();
            }
        }

        if self.speed == 0 {
            self.stop();
        }

        let waiting_time = timer_freq_micros / self.speed;
        unsafe {
            if CURRENT_TIME >= waiting_time {
                CURRENT_TIME = 0;
                IS_HIGH = !IS_HIGH;
            }
            CURRENT_TIME += 1;

            if IS_HIGH {
                self.clk.set_high().unwrap_or_default();
            } else {
                self.clk.set_low().unwrap_or_default();
            }
        }
    }

    fn stop(&mut self) {
        self.clk.set_low().unwrap_or_default();
        return;
    }

    pub fn set_speed(&mut self, speed: u32) {
        self.speed = speed;
    }

    pub fn new(dir_pin: T, clk: U, initial_speed: u32) -> Self {
        Self {
            dir_pin,
            clk,
            dir: Direction::default(),
            speed: initial_speed,
        }
    }
}
