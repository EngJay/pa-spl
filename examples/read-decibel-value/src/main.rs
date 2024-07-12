// Reads the latest decibel value and prints it to UART4.

#![no_main]
#![no_std]

use cortex_m::asm;
use cortex_m_rt::entry;

// Use halt as the panicking behavior.
//
// A breakpoint can be set on `rust_begin_unwind` to catch panics.
//
use panic_halt as _;
use pcb_artists_spl::PaSpl;
use stm32f3xx_hal::{delay::Delay, i2c::I2c, pac, prelude::*, serial::config, serial::Serial};

use core::fmt::Write;

struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufWriter<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        BufWriter { buf, pos: 0 }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.pos]).unwrap()
    }
}

impl<'a> core::fmt::Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let len = bytes.len();

        if self.pos + len > self.buf.len() {
            return Err(core::fmt::Error);
        }

        self.buf[self.pos..self.pos + len].copy_from_slice(bytes);
        self.pos += len;
        Ok(())
    }
}

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
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze(&mut flash.acr);

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

    // Use the I2C1 instance to create an instance of PaSpl.
    //
    let mut pa_spl = PaSpl::new(i2c);

    // Create a buffer able to be converted to a string.
    let mut buffer: [u8; 9] = [0; 9];
    let mut buf_writer = BufWriter::new(&mut buffer);

    // Delay in milliseconds between UART writes.
    //
    const UART_WRITE_DELAY_MS: u16 = 500;

    loop {
        // Get SPL value from the sensor, then format and convert it.
        //
        let spl = pa_spl.get_latest_decibel().unwrap();
        let mut buf_writer = BufWriter::new(&mut buffer);
        write!(buf_writer, "SPL: {}\r\n", spl).unwrap();
        let spl_str = core::str::from_utf8(&buffer).unwrap();

        // Write out to the UART.
        //
        uart4.write_str(spl_str).unwrap_or_else(|_| {
            loop {
                // Failed to write to UART4.
                asm::nop(); // If real app, replace with actual error handling.
            }
        });

        // Limit algorithm to (1000 * (1 / UART_WRITE_DELAY_MS)) Hz.
        delay.delay_ms(UART_WRITE_DELAY_MS);
    }
}
