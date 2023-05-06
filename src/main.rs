/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![no_std]
#![no_main]
use bsp::entry;
use defmt::*;
use defmt_rtt as _;

use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    multicore::{Multicore, Stack},
    pac,
    pac::interrupt,
    sio::Sio,
    timer,
    watchdog::Watchdog,
};

use cortex_m::delay::Delay;

use core::cell::RefCell;
use critical_section::Mutex;

use crate::drivers::StepperWithDriver;

mod drivers;

static mut CORE1_STACK: Stack<4096> = Stack::new();


type LedAndAlarm = (
    hal::gpio::Pin<hal::gpio::bank0::Gpio25, hal::gpio::PushPullOutput>,
    hal::timer::Alarm0,
);



#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    // let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
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

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead. If you have
    // a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here.
    let clk_pin = pins.led.into_push_pull_output();
    let dir_pin = pins.gpio11.into_push_pull_output();
    let speed = 2;

    // Setup core 2 for stepper motor
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);

    let sys_freq = clocks.system_clock.freq().to_Hz();

    let cores = mc.cores();

    let core1 = &mut cores[1];
    
    
    
    
    core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            // Get the second core's copy of the `CorePeripherals`, which are per-core.
            // Unfortunately, `cortex-m` doesn't support this properly right now,
            // so we have to use `steal`.
            let core = unsafe { pac::CorePeripherals::steal() };
            // Set up the delay for the second core.
            let mut delay = Delay::new(core.SYST, sys_freq);
            // let steps = 10; // TODO
            let mut stepper = StepperWithDriver::new(dir_pin, clk_pin, speed, 0);
            loop {
                let delay_ = |t: u32| delay.delay_ms(t);
                stepper.steps(delay_);
            }
        })
        .unwrap();
        
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0);
    }
    info!("on!");
    loop {
        cortex_m::asm::wfi();
    }
}

#[allow(non_snake_case)]
#[interrupt]
fn TIMER_IRQ_0() {}

// End of file
