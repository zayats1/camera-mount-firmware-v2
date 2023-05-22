/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(unused)]

use embedded_hal::digital::v2::OutputPin;

#[derive(Default)]
pub enum Directions {
    #[default]
    Forward,
    Backward,
    Stop,
}

pub struct StepperWithDriver<T: OutputPin, U: OutputPin> {
    dir_pin: T,
    clk: U,
    dir: Directions,
    speed: u32,
}

impl<T, U> StepperWithDriver<T, U>
where
    T: OutputPin,
    U: OutputPin,
{
    pub fn set_dir(&mut self, dir: Directions) {
        self.dir = dir
    }

    pub fn steps<F: FnMut(u32)>(&mut self, mut delay_ms: F) {
        match self.dir {
            Directions::Forward => {
                self.dir_pin.set_high().unwrap_or_default();
            }
            Directions::Backward => {
                self.dir_pin.set_low().unwrap_or_default();
            }
            Directions::Stop => {
                self.clk.set_low().unwrap_or_default();
                return;
            }
        }
        let delay_time = 1000 / self.speed;
        self.clk.set_high().unwrap_or_default();
        delay_ms(delay_time); // for prototype
        self.clk.set_low().unwrap_or_default();
        delay_ms(delay_time);
    }

    pub fn steps_with_timer(&mut self, timer_freq_micros: u32) {
        static mut IS_HIGH: bool = true;
        static mut CURRENT_TIME: u32 = 0;
        match self.dir {
            Directions::Forward => {
                self.dir_pin.set_high().unwrap_or_default();
            }
            Directions::Backward => {
                self.dir_pin.set_low().unwrap_or_default();
            }
            Directions::Stop => {
                self.clk.set_low().unwrap_or_default();
                return;
            }
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

    pub fn set_speed(&mut self, speed: u32) {
        self.speed = speed;
    }

    pub fn new(dir_pin: T, clk: U, speed: u32) -> Self {
        Self {
            dir_pin,
            clk,
            dir: Directions::default(),
            speed,
        }
    }
}
