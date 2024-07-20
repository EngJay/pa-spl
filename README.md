# PCB Artists I2C SPL Module Driver

![Build Status](https://github.com/engjay/pcb-artists-i2c-spl-module-driver-rust/actions/workflows/ci.yaml/badge.svg)
<!-- [![Latest Version]][crates.io] [![pa_spl msrv]][Rust 1.31] [![pa_spl msrv]][Rust 1.56] -->

*A Rust `no_std` [embedded-hal](https://github.com/rust-embedded/embedded-hal) driver for the
[PCB Artists I2C Sound Level module](https://pcbartists.com/product/i2c-decibel-sound-level-meter-module/).*

- [Example](examples/read-decibel-value/README.md) (STM32F3 Discovery)
- API Documentation - TODO
- [Vendor Documentation](docs/vendor/README.md)
- [Release Notes](https://github.com/EngJay/pcb-artists-i2c-spl-module-driver-rust/releases)

## Usage

<details>
<summary>
Click to show Cargo.toml.
<p></p>
</summary>

```toml
[dependencies]
pa-spl = 0.1
```

</details>

```rust
#![no_std]
#![no_main]

// ... board-specific includes ...
use pa_spl::{Error, PaSpl};

#[entry]
fn main() -> ! {
  // ... configuration + initialization for your board + I2C ...

  // Create an instance of PaSpl.
  //
  let mut pa_spl = PaSpl::new(i2c);

  // Get SPL value from the sensor.
  //
  let spl = pa_spl.get_latest_decibel().unwrap();
}
```

See the [example project](examples/read-decibel-value/README.md) for
a complete example for a STM32F3 Discovery.

## Features

- Read SPL value.
- Set window for averaging of SPL value over time.
- Set thresholds for triggering of interrupt pin that can be used as an external
  trigger.

## Rust Version Support

TODO

## Embedded HAL Version Support

TODO

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>
<br>
<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
