/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};

use critical_section::Mutex;
//use critical_section::Mutex;

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
    timer::{Alarm, Alarm0},
    uart::{self, DataBits, StopBits, UartConfig},
    watchdog::Watchdog,
    Timer,
};

use bsp::entry;

use fugit::{MicrosDurationU32, RateExtU32};

use crate::{
    drivers::{Servo, StepperWithDriver},
    parser::parse_data,
};

mod drivers;
mod parser;
mod tests;

type UartPins = (
    gpio::Pin<gpio::bank0::Gpio16, gpio::Function<gpio::Uart>>,
    gpio::Pin<gpio::bank0::Gpio17, gpio::Function<gpio::Uart>>,
);

type Reader = uart::Reader<pac::UART0, UartPins>;

static GLOBAL_READER: Mutex<RefCell<Option<Reader>>> = Mutex::new(RefCell::new(None));

static mut DATA_Q: Queue<u8, MESSAGE_BUFFER_SIZE> = Queue::<u8, MESSAGE_BUFFER_SIZE>::new();
static mut MESSAGE_Q: Queue<Message, 2> = Queue::new();
static mut IS_DATA_READY: AtomicBool = AtomicBool::new(false);

static mut CORE1_STACK: Stack<4096> = Stack::new();

static mut ALARM: Mutex<RefCell<Option<Alarm0>>> = Mutex::new(RefCell::new(None));
static mut IS_TICKED: AtomicBool = AtomicBool::new(false);

const TIMER_FREQ_MS: u32 = 1000;
const TIMER_FREQ: MicrosDurationU32 = MicrosDurationU32::millis(TIMER_FREQ_MS / 1000u32);

const SERVO_DUTY_ON_ZERO: u16 = 1640;
const SERVO_MAX_ANGLE: u16 = 180;

const STEPPER_MOTOR_INITIAL_SPEED: u32 = 5;

#[entry]
fn main() -> ! {
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
        // UART TX (characters sent from RP2040) on
        pins.gpio16.into_mode::<gpio::FunctionUart>(),
        // UART RX (characters received by RP2040)
        pins.gpio17.into_mode::<gpio::FunctionUart>(),
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

    //Setup stepper motor
    // let clk_pin = pins.led.into_push_pull_output(); // gpio 15 in real world
    let clk_pin = pins.gpio15.into_push_pull_output();
    let dir_pin = pins.gpio11.into_push_pull_output();
     //let dir_pin = pins.led.into_push_pull_output();

    let mut stepper = StepperWithDriver::new(dir_pin, clk_pin, STEPPER_MOTOR_INITIAL_SPEED);

    // Setup second core for parsing data
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);

    let cores = mc.cores();

    let core1 = &mut cores[1];

    let mut pwm_slices = pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // Servo init

    let pwm = &mut pwm_slices.pwm1;
    pwm.set_ph_correct();
    pwm.set_div_int(20u8); // 50 hz   1/50= 0.020 s
    pwm.enable();

    let channel = &mut pwm.channel_b;
    channel.output_to(pins.gpio3);

    let mut servo = Servo::new(channel, SERVO_DUTY_ON_ZERO, SERVO_MAX_ANGLE);

    // Timer init
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    critical_section::with(|cs| {
        let mut alarm = timer.alarm_0().unwrap();
        let _ = alarm.schedule(TIMER_FREQ);
        alarm.enable_interrupt();
        // Move alarm into ALARM, so that it can be accessed from interrupts
        unsafe {
            ALARM.borrow(cs).replace(Some(alarm));
        }
    });

    unsafe {
        // Enable the UART interrupt in the *Nested Vectored Interrupt
        // Controller*, which is part of the Cortex-M0+ core.
        pac::NVIC::unmask(pac::Interrupt::UART0_IRQ);

        // Unmask the timer0 IRQ so that it will generate an interrupt
        pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
    }

    core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            let mut message_producer = unsafe { MESSAGE_Q.split().0 };
            let mut data_consumer = unsafe { DATA_Q.split().1 };

            loop {
                let is_data_ready = unsafe { IS_DATA_READY.load(Ordering::Relaxed) };
                if is_data_ready {
                    unsafe {
                        IS_DATA_READY.store(false, Ordering::Relaxed);
                    }

                    // the code runs on the MCU and in debug  mode only
                    #[cfg(debug_assertions)]
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
                } else {
                    cortex_m::asm::wfe();
                }
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

        let is_ticked = unsafe { IS_TICKED.load(Ordering::Relaxed) };
        if is_ticked {
            unsafe {
                IS_TICKED.store(false, Ordering::Relaxed);
            }
            stepper.steps_with_timer(TIMER_FREQ_MS);
        }

        cortex_m::asm::wfe();
    }
}

#[interrupt]
fn UART0_IRQ() {
    static mut READER: Option<Reader> = None;

    let mut producer = unsafe { DATA_Q.split().0 };

    if READER.is_none() {
        critical_section::with(|cs| {
            *READER = GLOBAL_READER.borrow(cs).take();
        });
    }

    // Check if we have a UART to work with
    if let Some(reader) = READER {
        unsafe {
            IS_DATA_READY.store(true, Ordering::Relaxed);
        }
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

#[allow(non_snake_case)]
#[interrupt]
fn TIMER_IRQ_0() {
    critical_section::with(|cs| {
        // Temporarily take our ALARM
        let alarm = unsafe { ALARM.borrow(cs).take() };
        if let Some(mut alarm) = alarm {
            alarm.clear_interrupt();
            let _ = alarm.schedule(TIMER_FREQ);
            unsafe {
                IS_TICKED.store(true, Ordering::Relaxed);
            }
            // Return ALARM into our static variable
            unsafe {
                ALARM.borrow(cs).replace_with(|_| Some(alarm));
            }
        }
    });
    // Set an event to ensure the main thread always wakes up, even if it's in
    // the process of going to sleep.
    cortex_m::asm::sev();
}
