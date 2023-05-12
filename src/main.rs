/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;

use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    dma,
    dma::DMAExt,
    gpio,
    multicore::{Multicore, Stack},
    pac,
    sio::Sio,
    uart,
    watchdog::Watchdog,
};

use bsp::{
    entry,
    hal::uart::{DataBits, StopBits, UartConfig},
};

use cortex_m::{delay::Delay, singleton};
use fugit::RateExtU32;

use crate::drivers::StepperWithDriver;

mod drivers;

static mut CORE1_STACK: Stack<4096> = Stack::new();

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

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead. If you have
    // a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here.
    let clk_pin = pins.led.into_push_pull_output();
    let dir_pin = pins.gpio11.into_push_pull_output();
    let speed = 2;

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

    let dma = pac.DMA.split(&mut pac.RESETS);

    uart.write_full_blocking(b"\r\n\r\nUART DMA echo example\r\n\r\n");

    // In order to use DMA we need to split the UART into a RX (receive) and TX (transmit) pair
    let (rx, tx) = uart.split();

    // We can still write to the tx side of the UART after splitting
    tx.write_full_blocking(b"Regular UART write\r\n");

    // And we can DMA from a buffer into the UART
    let teststring = b"DMA UART write\r\n";
    let tx_transfer = dma::single_buffer::Config::new(dma.ch0, teststring, tx).start();

    // Wait for the DMA transfer to finish so we can reuse the tx and the dma channel
    let (ch0, _teststring, tx) = tx_transfer.wait();

    // Let's test DMA RX into a buffer.
    tx.write_full_blocking(b"Waiting for you to type 5 letters...\r\n");
    let rx_buf = singleton!(: [u8; 5] = [0; 5]).unwrap();
    let rx_transfer = dma::single_buffer::Config::new(ch0, rx, rx_buf).start();
    let (ch0, rx, rx_buf) = rx_transfer.wait();

    // Echo back the 5 characters the user typed
    tx.write_full_blocking(b"You wrote \"");
    tx.write_full_blocking(rx_buf);
    tx.write_full_blocking(b"\"\r\n");

    // Now just keep echoing anything that is received back out of TX
    tx.write_full_blocking(b"Now echoing any character you write...\r\n");
    let _tx_transfer = dma::single_buffer::Config::new(ch0, rx, tx).start();

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

    loop {
        cortex_m::asm::wfe();
    }
}

// End of file
