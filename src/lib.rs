#![cfg_attr(not(test), no_std)]

use embedded_hal::i2c::I2c;

/// PCB Artists SPL Module I2C address.
const DEVICE_ADDR: u8 = 0x48;

/// PCB Artists SPL Module Versions.
/// MEMS mic version, long term support and available for sale.
const DEVICE_VER_MEMS_LTS: u8 = 0x31;

/// DEVICE_VER_MEMS_LTS features + audio spectrum analyzer.
const DEVICE_VER_MEMS_LTS_ASA: u8 = 0x32;

/// External mic, long term support and available for sale.
const DEVICE_VER_MEMS_LTS_EXT: u8 = 0x81;

/// Device Registers.
const VER_REGISTER: u8 = 0x00;

/// A PA SPL Module on the I2C bus `I2C`.
pub struct PaSpl<I2C>
where
    I2C: I2c,
{
    i2c: Option<I2C>,
}

// / A driver error.
#[derive(Debug, PartialEq)]
pub enum Error<E> {
    /// I2C bus error.
    I2c(E),
    /// No I2C instance available.
    NoI2cInstance,
}

impl<I2C> PaSpl<I2C>
where
    I2C: I2c,
{
    /// Initializes the PA SPL Module driver.
    pub fn new(i2c: I2C) -> Self {
        Self { i2c: Some(i2c) }
    }

    pub fn read_firmware_version<E>(&mut self) -> Result<u8, Error<E>>
    where
        I2C: embedded_hal::i2c::I2c<Error = E>,
    {
        let mut ver = [0];

        if let Some(ref mut i2c) = self.i2c {
            i2c.write_read(DEVICE_ADDR, &[VER_REGISTER], &mut ver)
                .map_err(Error::I2c)?;
            Ok(ver[0])
        } else {
            // Return an appropriate error if I2C is None.
            Err(Error::NoI2cInstance)
        }
    }

    /// Destroys this driver and releases the I2C bus.
    pub fn destroy(mut self) -> I2C {
        self.i2c.take().expect("I2C instance already taken")
    }
}

impl<I2C> Drop for PaSpl<I2C>
where
    I2C: I2c,
{
    fn drop(&mut self) {
        if let Some(_i2c) = self.i2c.take() {
            // Additional clean-up, logging, etc. here.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PaSpl, DEVICE_ADDR, DEVICE_VER_MEMS_LTS_ASA, VER_REGISTER};
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    #[test]
    fn read_firmware_version() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR,
            vec![VER_REGISTER],
            vec![DEVICE_VER_MEMS_LTS_ASA],
        )];
        let i2c_mock = I2cMock::new(&expectations);

        let mut pa_spl = PaSpl::new(i2c_mock);
        let version = pa_spl.read_firmware_version().unwrap();
        assert_eq!(DEVICE_VER_MEMS_LTS_ASA, version);

        let mut mock = pa_spl.destroy();
        mock.done(); // Verify expectations.1
    }
}
