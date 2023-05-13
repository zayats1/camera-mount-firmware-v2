/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![no_std]
#![no_main]

//use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;

use heapless::spsc::Queue;
use panic_probe as _;

use parser::Message;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio,
    multicore::{Multicore, Stack},
    pac,
    // pac::interrupt,
    sio::Sio,
    uart::{self, DataBits, StopBits, UartConfig},
    watchdog::Watchdog,
};

use bsp::entry;

use cortex_m::delay::Delay;
use fugit::RateExtU32;

use crate::{
    drivers::StepperWithDriver,
    parser::{parse_data, MESSAGE_BUFFER_SIZE},
};

mod drivers;
mod parser;

static mut CORE1_STACK: Stack<4096> = Stack::new();

static mut MESSAGE_Q: Queue<Message, 2> = Queue::new();

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

    // let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

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
    let mut uart = uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    uart.enable_rx_interrupt();

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

    core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || loop {
            let mut producer = unsafe { MESSAGE_Q.split().0 };
            if uart.uart_is_readable() {
                let mut buf = [0u8; MESSAGE_BUFFER_SIZE];
                if let Ok(()) = uart.read_full_blocking(&mut buf) {
                    uart.write_full_blocking(&buf);
                }
                match parse_data(&buf) {
                    Ok(m) => {
                        let _ = producer.enqueue(m);
                    }
                    Err(e) => {
                        uart.write_full_blocking(e.describe().as_bytes());
                    }
                };
            }
        })
        .unwrap();

    loop {
        // cortex_m::asm::wfe();
        // uart.write_full_blocking(b"fuck");
        loop {
            let mut consumer = unsafe { MESSAGE_Q.split().1 };
            if let Some(message) = consumer.dequeue() {
                match message {
                    Message::StepperMotorSpeed(speed) => stepper.set_speed(speed),
                    Message::StepperMotorDir(dir) => stepper.set_dir(dir),
                    Message::ServoAngle(_) => {}
                }
            }

            let delay_ = |t: u32| delay.delay_ms(t);
            stepper.steps(delay_);
        }
    }
}
