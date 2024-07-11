// Reads the latest decibel value and prints it to UART?

#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Use halt as the panicking behavior.
//
// A breakpoint can be set on `rust_begin_unwind` to catch panics.
//
use panic_halt as _;

use cortex_m::asm;
use cortex_m_rt::entry;

use stm32f3xx_hal::{delay::Delay, i2c::I2c, pac, prelude::*, serial::config, serial::Serial};

#[entry]
fn main() -> ! {
    // Get peripherals.
    //
    // take() returns an Option, which requires handling the possibility of the
    // return of an Err or None instead of the desired value, which is of type
    // pac::Peripherals in this case.
    //
    // Since this is an embedded application, it's not as simple as writing to,
    // stdout. This is a minimal example, so we'll drop into an inifinite loop
    // to allow a debugger to find where the failure.
    //
    let device_periphs = pac::Peripherals::take().unwrap_or_else(|| {
        loop {
            // Failed to take cortex_m::Peripherals.
            asm::nop(); // If real app, replace with actual error handling.
        }
    });

    // Get RCC peripheral and configure clocks.
    //
    // The constrain() method is used here to provide a higher-level abstraction
    // of the peripheral rather than raw register access. The method consumes
    // the raw peripheral and returns an instance of the RCC peripheral with
    // higher-level safe abstractions provided by the HAL, which is of type Rcc,
    // while setting the system clock frequency.
    //
    let mut rcc = device_periphs.RCC.constrain();
    let mut flash = device_periphs.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Set up delay capability.
    //
    // Use the same unwrap method to get the core periphs, then
    // create a delay abstrction using SysTick (SYST).
    //
    let core_periphs = cortex_m::Peripherals::take().unwrap_or_else(|| {
        loop {
            // Failed to take cortex_m::Peripherals.
            asm::nop(); // If real app, replace with actual error handling.
        }
    });
    let mut delay = Delay::new(core_periphs.SYST, clocks);

    // Get GPIO Ports B and C.
    //
    // The split method here splits out the functionality of the GPIO Port B/C
    // while taking a mutable borrow of an "enabler" that enables the clock for
    // the port at the same time. The mutable borrow allows modification of the
    // borrowed value while ensuring exclusive access.
    //
    let mut gpiob = device_periphs.GPIOB.split(&mut rcc.ahb);
    let mut gpioc = device_periphs.GPIOC.split(&mut rcc.ahb);

    // Configure pins PB6 as SCL and PB7 as SDA for I2C1.
    //
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

    // Create an instance of I2C1 with the pins.
    //
    let i2c = I2c::new(
        device_periphs.I2C1,
        (scl, sda),
        100.kHz().try_into().unwrap(),
        clocks,
        &mut rcc.apb1,
    );

    // Configure GPIO pins PC10 as TX and PC11 as RX for UART4.
    //
    let tx_pin = gpioc
        .pc10
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
    let rx_pin = gpioc
        .pc11
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);

    // Create an instance of UART4 with the pins.
    //
    let mut uart4 = Serial::new(
        device_periphs.UART4,
        (tx_pin, rx_pin),
        config::Config::default().baudrate(115_200.Bd()),
        clocks,
        &mut rcc.apb1,
    );

    loop {}
}
