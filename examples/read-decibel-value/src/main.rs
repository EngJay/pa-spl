// Reads the latest decibel value and prints it to UART?

#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m::asm;
use cortex_m_rt::entry;

use stm32f3xx_hal::{delay::Delay, i2c::I2c, pac, prelude::*, serial::config, serial::Serial};

#[entry]
fn main() -> ! {
    // Initialize I2C.
    let device_periphs = pac::Peripherals::take().unwrap_or_else(|| {
        loop {
            // Failed to take cortex_m::Peripherals.
            asm::nop(); // If real app, replace with actual error handling.
        }
    });
    let mut rcc = device_periphs.RCC.constrain();
    let mut flash = device_periphs.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpiob = device_periphs.GPIOB.split(&mut rcc.ahb);

    // Configure I2C1.
    let mut scl =
        gpiob
            .pb6
            .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let mut sda =
        gpiob
            .pb7
            .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    scl.internal_pull_up(&mut gpiob.pupdr, true);
    sda.internal_pull_up(&mut gpiob.pupdr, true);
    let i2c = I2c::new(
        device_periphs.I2C1,
        (scl, sda),
        100.kHz().try_into().unwrap(),
        clocks,
        &mut rcc.apb1,
    );

    loop {}
}
