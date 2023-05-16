#![allow(unused)]

use crate::drivers::stepper_with_driver::Direction;

pub const MESSAGE_BUFFER_SIZE: usize = 5;

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

pub fn parse_data(data: &[u8]) -> Result<Message, ParseDataError> {
    for (i, character) in data.iter().enumerate() {
        let parsed = match *character {
            ComCodePrefixes::SERVO_ANGLE => {
                let parsed_digits = parse_digits(data, i);
                if let Some(angle) = parsed_digits {
                    Some(Message::ServoAngle(angle))
                } else {
                    None
                }
            }

            ComCodePrefixes::STEPPER_MOTOR_DIR => {
                Some(Message::StepperMotorDir(Direction::Forward))
            }
            ComCodePrefixes::STEPPER_MOTOR_SPEED => {
                let parsed_digits = parse_digits(data, i);
                if let Some(speed) = parsed_digits {
                    Some(Message::StepperMotorSpeed(speed.into()))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(message) = parsed {
            return Ok(message);
        }
    }

    Err(ParseDataError::new())
}

fn parse_digits(data: &[u8], index: usize) -> Option<u16> {
    const DIGIT_COUNT: u16 = 3;
    let mut digits = [0u8; DIGIT_COUNT as usize];
    for (j, digit) in digits.iter_mut().enumerate() {
        let aquired_diget = data.get(index + 1 + j);
        if let Some(num) = aquired_diget {
            *digit = *num;
        } else {
            return None;
        }
    }
    let mut decimal_place = 10u16.pow(DIGIT_COUNT as u32);
    let parsed = digits
        .into_iter()
        .map(|ch| {
            decimal_place /= 10;
            (ch - 48) as u16 * decimal_place
        })
        .sum::<u16>();
    return Some(parsed);
}
