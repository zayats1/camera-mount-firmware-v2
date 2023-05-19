/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![no_std]
#![no_main]

use core::cell::RefCell;

use critical_section::Mutex;
//use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;

use embedded_hal::serial::{Read, Write};
use heapless::spsc::Queue;
use panic_probe as _;

use parser::{Message, MESSAGE_BUFFER_SIZE};
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio,
    multicore::{Multicore, Stack},
    pac,
    pac::interrupt,
    pwm,
    sio::Sio,
    uart::{self, DataBits, StopBits, UartConfig},
    watchdog::Watchdog,
};

use bsp::entry;

use cortex_m::delay::Delay;
use fugit::RateExtU32;

use crate::{
    drivers::{Servo, StepperWithDriver},
    parser::parse_data,
};

mod drivers;
mod parser;
mod tests;

type UartPins = (
    gpio::Pin<gpio::bank0::Gpio0, gpio::Function<gpio::Uart>>,
    gpio::Pin<gpio::bank0::Gpio1, gpio::Function<gpio::Uart>>,
);

type Reader = uart::Reader<pac::UART0, UartPins>;

static GLOBAL_READER: Mutex<RefCell<Option<Reader>>> = Mutex::new(RefCell::new(None));

static mut CORE1_STACK: Stack<4096> = Stack::new();

static mut DATA_Q: Queue<u8, MESSAGE_BUFFER_SIZE> = Queue::<u8, MESSAGE_BUFFER_SIZE>::new();

static mut MESSAGE_Q: Queue<Message, 2> = Queue::new();

const SERVO_DUTY_ON_ZERO: u16 = 1640;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();

    // let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (
        // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
        pins.gpio0.into_mode::<gpio::FunctionUart>(),
        // UART RX (characters received by RP2040) on pin 2 (GPIO1)
        pins.gpio1.into_mode::<gpio::FunctionUart>(),
    );

    // Make a UART on the given pins
    let uart = uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let (mut rx, mut tx) = uart.split();

    rx.enable_rx_interrupt();

    critical_section::with(|cs| {
        GLOBAL_READER.borrow(cs).replace(Some(rx));
    });

    // the code runs on the MCU and in debug   modeonly
    #[cfg(debug_assertions)]
    {
        let mut tester = tests::UnitTest::new(&mut tx);
        tester.run_tests();
    }

    let core = pac::CorePeripherals::take().unwrap();
    let sys_freq = clocks.system_clock.freq().to_Hz();

    // Set up the delay for  core0.
    let mut delay = Delay::new(core.SYST, sys_freq);

    let clk_pin = pins.led.into_push_pull_output();
    let dir_pin = pins.gpio11.into_push_pull_output();
    // let steps = 10; // TODO
    let speed = 2;
    let mut stepper = StepperWithDriver::new(dir_pin, clk_pin, speed, 0);

    // Setup core 2 for parsing data
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);

    let cores = mc.cores();

    let core1 = &mut cores[1];

    let mut pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    let pwm = &mut pwm_slices.pwm1;
    pwm.set_ph_correct();
    pwm.set_div_int(20u8); // 50 hz   1/50= 0.020 s
    pwm.enable();

    // Servo
    let channel = &mut pwm.channel_b;
    channel.output_to(pins.gpio3);

    let mut servo = Servo::new(channel, SERVO_DUTY_ON_ZERO);

    unsafe {
        // Enable the UART interrupt in the *Nested Vectored Interrupt
        // Controller*, which is part of the Cortex-M0+ core.
        pac::NVIC::unmask(pac::Interrupt::UART0_IRQ);
    }

    // TODO fix blocking while buffer isn`t full
    core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            let mut message_producer = unsafe { MESSAGE_Q.split().0 };
            let mut data_consumer = unsafe { DATA_Q.split().1 };

            loop {
                cortex_m::asm::wfe();
                unsafe {
                    for data in DATA_Q.into_iter() {
                        let _ = tx.write(*data);
                    }
                }
                match parse_data(&mut data_consumer) {
                    Ok(m) => {
                        let _ = message_producer.enqueue(m);
                    }
                    Err(e) => {
                        tx.write_full_blocking(e.describe().as_bytes());
                    }
                };
            }
        })
        .unwrap();

    let mut consumer = unsafe { MESSAGE_Q.split().1 };
    loop {
        if let Some(message) = consumer.dequeue() {
            match message {
                Message::StepperMotorSpeed(speed) => stepper.set_speed(speed),
                Message::StepperMotorDir(dir) => stepper.set_dir(dir),
                Message::ServoAngle(angle) => servo.set_angle(angle),
            }
        }

        let delay_ = |t: u32| delay.delay_ms(t);
        stepper.steps(delay_);
    }
}

#[interrupt]
fn UART0_IRQ() {
    // Set an event to ensure the main thread always wakes up, even if it's in
    // the process of going to sleep.
    static mut READER: Option<Reader> = None;

    let mut producer = unsafe { DATA_Q.split().0 };
    // This is one-time lazy initialisation. We steal the variable given to us
    // via `GLOBAL_UART`.
    if READER.is_none() {
        critical_section::with(|cs| {
            *READER = GLOBAL_READER.borrow(cs).take();
        });
    }

    // Check if we have a UART to work with
    if let Some(reader) = READER {
        while let Ok(byte) = reader.read() {
            if producer.enqueue(byte).is_err() {
                break;
            }
        }
    }

    // Set an event to ensure the main thread always wakes up, even if it's in
    // the process of going to sleep.

    cortex_m::asm::sev();
}
