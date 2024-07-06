#![cfg_attr(not(test), no_std)]

use embedded_hal::i2c::I2c;

/// PCB Artists SPL Module I2C address.
const DEVICE_ADDR: u8 = 0x48;

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
