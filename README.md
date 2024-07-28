# Embedded HAL Driver for PCB Artists I2C SPL Module

![Build Status](https://github.com/engjay/pa-spl/actions/workflows/ci.yaml/badge.svg)

*A Rust `no_std` [embedded-hal](https://github.com/rust-embedded/embedded-hal) driver for the
[PCB Artists I2C Sound Level module](https://pcbartists.com/product/i2c-decibel-sound-level-meter-module/).*

- [Example](https://github.com/EngJay/pa-spl/blob/main/examples/read-decibel-value/README.md)
  ([STM32F3 Discovery](https://www.st.com/en/evaluation-tools/stm32f3discovery.html))
- API Documentation <!-- TODO -->
- [Vendor Documentation](docs/vendor/README.md)
- [Release Notes](https://github.com/EngJay/pa-spl/releases)

## Features

- Read current SPL value averaged over TAVG with a range of 35 dB to 120 dB
  (+/-2 dB) from 30 Hz to 8 kHz.
- Adjustable TAVG window for averaging of SPL value from 10 ms to 10,000 ms.
- Read min/max SPL value sensed between power cycle or reset.

## Usage

This example uses the SPL module with a STM32F3 Discovery development board and a USB-TTL converter.

See the
[example project](https://github.com/EngJay/pa-spl/blob/main/examples/read-decibel-value/README.md)
for the complete example.

<details>
<summary>
Click to show Cargo.toml.
<p></p>
</summary>

```toml
[package]
name = "example-read-decibel-value"
description = "Example of reading SPL with the pa-spl driver, PCB Artists SPL module, and STM32F3 Discovery"
authors = ["Jason Scott <>"]
edition = "2021"
publish = false
readme = "README.md"
version = "0.1.0"

[dependencies]
cortex-m = "0.7.7"
cortex-m-rt = "0.7.3"
cortex-m-semihosting = "0.5.0"
panic-halt = "0.2.0"
stm32f3xx-hal = { version = "0.10.0", features = ["ld", "rt", "stm32f303xc"] }
pa-spl = "0.1.0"

[[bin]]
name = "example-read-decibel-value"
test = false
bench = false

[profile.release]
codegen-units = 1
debug = true
lto = true
```

</details>

```rust,ignore
// Reads the latest decibel value and prints it to UART4.

#![no_main]
#![no_std]

use cortex_m::asm;
use cortex_m_rt::entry;

// Use halt as the panicking behavior.
//
// A breakpoint can be set on `rust_begin_unwind` to catch panics.
//
use pa_spl::PaSpl;
use panic_halt as _;
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

    pub fn reset(&mut self) {
        self.pos = 0;
        self.buf.fill(0);
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
    let mut buffer: [u8; 8] = [0; 8];
    let mut buf_writer = BufWriter::new(&mut buffer);

    // Delay in milliseconds between UART writes.
    //
    const UART_WRITE_DELAY_MS: u16 = 500;

    loop {
        // Reset the buffer at the start of each iteration
        //
        buf_writer.reset();

        // Get SPL value from the sensor.
        //
        let spl = pa_spl.get_latest_decibel().unwrap();

        // Format string with SPL value, then covert to string.
        //
        write!(buf_writer, "SPL: {}\r", spl).unwrap();
        let spl_str = buf_writer.as_str();

        // Write the string out to the UART.
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

```

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust
[1.65](https://releases.rs/docs/1.60.0/) and up. It might compile with older
versions but that may change in any new patch release.

See [here](https://github.com/EngJay/pa-spl/tree/main/docs/msrv.md) for details
on how the MSRV may be upgraded.

## Minimum Supported Embedded HAL Version

TL;DR: This initial release only supports embedded-hal 0.2 and support for the
1.0 version will be added in a subsequent release.

This crate depends on the [embedded-hal](https://crates.io/crates/embedded-hal)
crate as it is a driver for use with embedded-hal. Embedded versioning typically
moves significantly slower than mainstream, so numerous crates in the repository
still depend on the [0.2](https://crates.io/crates/embedded-hal/0.2.7) version
of embedded-hal rather than the recent 1.0 release. Due to this, the minimum
supported embedded-hal version of this crate is 0.2 and the 1.0 version is not
yet supported in this initial release. Support for the 1.0 version of
embedded-hal will be added in a subsequent release.

#### License

<sup>
Licensed under either of <a href="https://github.com/EngJay/pa-spl/blob/main/LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="https://github.com/EngJay/pa-spl/blob/main/LICENSE-MIT">MIT license</a> at your option.
</sup>
<br>
<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>

#### Contribution

<sup>
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
</sup>
