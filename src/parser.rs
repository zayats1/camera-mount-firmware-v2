/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::drivers::stepper_with_driver::Direction;
use heapless::spsc::Consumer;

pub const MESSAGE_BUFFER_SIZE: usize = 8;

#[derive(PartialEq)]
pub enum Message {
    StepperMotorSpeed(u32),
    StepperMotorDir(Direction),
    ServoAngle(u16),
}

pub enum ComCodePrefixes {}

impl ComCodePrefixes {
    pub const STEPPER_MOTOR_SPEED: u8 = b'S';
    pub const SERVO_ANGLE: u8 = b'A';
    pub const STEPPER_MOTOR_DIR: u8 = b'D';
}

pub enum StepperMotorDir {}

impl StepperMotorDir {
    pub const FORWARD: u8 = b'F';
    pub const BACKWARDS: u8 = b'B';
    pub const STOP: u8 = b'S';
}

#[derive(PartialEq)]
pub struct ParseDataError {
    description: &'static str,
}
impl ParseDataError {
    pub fn new() -> Self {
        Self {
            description: "This data can't be parsed \n",
        }
    }
    pub fn describe(&self) -> &'static str {
        self.description
    }
}

pub fn parse_data(
    consumer: &mut Consumer<u8, MESSAGE_BUFFER_SIZE>,
) -> Result<Message, ParseDataError> {
    while let Some(character) = consumer.dequeue() {
        let parsed = match character {
            ComCodePrefixes::SERVO_ANGLE => {
                let parsed_digits = parse_digits(consumer);
                parsed_digits.map(Message::ServoAngle)
            }

            ComCodePrefixes::STEPPER_MOTOR_DIR => {
                let parsed_char = consumer.dequeue();
                let mut dir = None;
                if let Some(parsed_char) = parsed_char {
                    dir = match parsed_char {
                        StepperMotorDir::FORWARD => Some(Direction::Forward),
                        StepperMotorDir::BACKWARDS => Some(Direction::Backward),
                        StepperMotorDir::STOP => Some(Direction::Stop),
                        _ => None,
                    }
                }
                dir.map(Message::StepperMotorDir)
            }
            ComCodePrefixes::STEPPER_MOTOR_SPEED => {
                let parsed_digits = parse_digits(consumer);
                parsed_digits.map(|speed| Message::StepperMotorSpeed(speed.into()))
            }
            _ => None,
        };
        if let Some(message) = parsed {
            return Ok(message);
        }
    }

    Err(ParseDataError::new())
}

fn parse_digits(consumer: &mut Consumer<u8, MESSAGE_BUFFER_SIZE>) -> Option<u16> {
    const DIGIT_COUNT: u16 = 3;

    let mut decimal_place = DIGIT_COUNT;
    let mut digits = [0u16; DIGIT_COUNT as usize];

    for digit in digits.iter_mut() {
        let aquired_diget = consumer.dequeue();
        if let Some(num) = aquired_diget {
            let is_num = b'0' <= num && num <= b'9';
            if is_num {
                decimal_place -= 1;
                *digit = (num - b'0') as u16 * 10u16.pow(decimal_place as u32);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    let sum = digits.into_iter().sum();

    Some(sum)
}
