// standart rust testings doesnt work, so I should use mine instead
use embedded_hal::serial::Write;

use crate::parser::{parse_data, Message, MESSAGE_BUFFER_SIZE};
use heapless::spsc::Queue;
pub struct UnitTest<'a, T: Write<u8>> {
    logger: &'a mut T,
}

impl<'a, T: Write<u8>> UnitTest<'a, T> {
    fn assert(&mut self, is_the_test_passed: bool) {
        if is_the_test_passed {
            self.write(b"PASSED \n");
        } else {
            self.write(b"FAILED \n");
            panic!();
        };
    }

    fn assert_eq<F: PartialEq>(&mut self, expected: F, obtained: F) {
        self.assert(expected == obtained);
    }

    fn write(&mut self, res: &[u8]) {
        for byte in res {
            let _ = self.logger.write(*byte);
        }
    }
    pub fn new(logger: &'a mut T) -> Self {
        Self { logger }
    }
    // add tests here
    fn parsing_test(&mut self) {
        let data = "A090\n".as_bytes();
        let mut queue = Queue::<u8, MESSAGE_BUFFER_SIZE>::new();

        let (mut producer, mut consumer) = queue.split();

        for byte in data {
            if let Err(_) = producer.enqueue(*byte) {
                break;
            }
        }

        self.assert_eq(parse_data(&mut consumer), Ok(Message::ServoAngle(90u16)));
    }

    pub fn run_tests(&mut self) {
        //call the tests here
        self.parsing_test();
    }
}
