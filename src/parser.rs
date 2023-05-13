#![allow(unused)]

use crate::drivers::stepper_with_driver::Direction;

pub const MESSAGE_BUFFER_SIZE: usize = 5;

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

pub struct ParseDataError {
    description: &'static str,
}
impl ParseDataError {
    pub fn new() -> Self {
        Self {
            description: "This data can't be parsed",
        }
    }
    pub fn describe(&self) -> &'static str {
        self.description
    }
}

pub fn parse_data(data: &[u8]) -> Result<Message, ParseDataError> {
    match data[0] {
        ComCodePrefixes::SERVO_ANGLE => Ok(Message::ServoAngle(90u16)),
        ComCodePrefixes::STEPPER_MOTOR_DIR => Ok(Message::StepperMotorDir(Direction::Forward)),
        ComCodePrefixes::STEPPER_MOTOR_SPEED => Ok(Message::StepperMotorSpeed(5)),
        _ => Err(ParseDataError::new()),
    }
}
