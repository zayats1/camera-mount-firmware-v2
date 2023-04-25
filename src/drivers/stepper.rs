/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(unused)]

#[derive(Default)]
pub enum Directions {
    #[default]
    Forward,
    Backward,
}

pub trait Stepper {
    fn steps<F: FnMut(u32)>(&mut self, steps: usize, delay: F);
    fn stop(&mut self);
    fn hold(&mut self);
    fn set_dir(&mut self, dir: Directions);
    fn set_speed(&mut self, speed: u32);
    fn set_acceleration(&mut self, acceleration: i32);
}
