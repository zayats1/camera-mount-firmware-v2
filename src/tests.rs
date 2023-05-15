// standart rust testings doesnt work, so I should use mine instead
use embedded_hal::serial::Write;
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
        self.assert(true);
    }

    pub fn test(&mut self) {
        //call the tests here
        self.parsing_test();
    }
}
