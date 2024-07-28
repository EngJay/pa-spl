# Driver for PCB Artists I2C SPL Module

![Build Status](https://github.com/engjay/pcb-artists-i2c-spl-module-driver-rust/actions/workflows/ci.yaml/badge.svg)

*A Rust `no_std` [embedded-hal](https://github.com/rust-embedded/embedded-hal) driver for the
[PCB Artists I2C Sound Level module](https://pcbartists.com/product/i2c-decibel-sound-level-meter-module/).*

- [Example](https://github.com/EngJay/pa-spl/blob/main/examples/read-decibel-value/README.md) (STM32F3 Discovery)
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

```rust,ignore
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

See the
[example project](https://github.com/EngJay/pa-spl/blob/main/examples/read-decibel-value/README.md)
for a complete example for a STM32F3 Discovery.

## Features

- Read SPL value.
- Set window for averaging of SPL value over time.
- Set thresholds for triggering of interrupt pin that can be used as an external
  trigger.

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
