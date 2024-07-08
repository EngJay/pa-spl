#![cfg_attr(not(test), no_std)]

use embedded_hal::blocking::i2c;

/// PCB Artists SPL Module I2C address.
const DEVICE_ADDR: u8 = 0x48;

/// PCB Artists SPL Module Versions.
// /// MEMS mic version, long term support and available for sale.
// const DEVICE_VER_MEMS_LTS: u8 = 0x31;

// /// DEVICE_VER_MEMS_LTS features + audio spectrum analyzer.
// const DEVICE_VER_MEMS_LTS_ASA: u8 = 0x32;

// /// External mic, long term support and available for sale.
// const DEVICE_VER_MEMS_LTS_EXT: u8 = 0x81;

/// Device Registers.
const VER_REGISTER: u8 = 0x00;
const SCRATCH_REGISTER: u8 = 0x05;
const DEVICE_ID_REGISTERS: [u8; 4] = [0x01, 0x02, 0x03, 0x04]; // ID3, ID2, ID1, ID0

/// A PA SPL Module on the I2C bus `I`.
pub struct PaSpl<I2C>
where
    I2C: i2c::Read + i2c::Write + i2c::WriteRead,
{
    i2c: Option<I2C>,
}

/// A driver error.
#[derive(Debug, PartialEq)]
pub enum Error<E> {
    /// I2C bus error.
    I2c(E),
    /// No I2C instance available.
    NoI2cInstance,
}

impl<E, I2C> PaSpl<I2C>
where
    I2C: i2c::Read<Error = E> + i2c::Write<Error = E> + i2c::WriteRead<Error = E>,
{
    /// Initializes the PA SPL Module driver.
    pub fn new(i2c: I2C) -> Self {
        Self { i2c: Some(i2c) }
    }

    pub fn get_device_id(&mut self) -> Result<u32, Error<E>> {
        let mut buffer: [u8; 4] = [0; 4];
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write_read(DEVICE_ADDR, &[DEVICE_ID_REGISTERS[0]], &mut buffer)
            .map_err(Error::I2c)?;

        // Combine the bytes into a u32.
        let device_id: u32 = ((buffer[0] as u32) << 24)
            | ((buffer[1] as u32) << 16)
            | ((buffer[2] as u32) << 8)
            | (buffer[3] as u32);

        Ok(device_id)
    }

    pub fn get_firmware_version(&mut self) -> Result<u8, Error<E>> {
        let mut buffer = [0; 1];
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write_read(DEVICE_ADDR, &[VER_REGISTER], &mut buffer)
            .map_err(Error::I2c)?;

        Ok(buffer[0])
    }

    /// Destroys this driver and releases the I2C bus.
    pub fn destroy(mut self) -> I2C {
        self.i2c
            .take()
            .expect("I2C instance has already been taken")
    }
}

#[cfg(test)]
mod tests {
    use super::{PaSpl, DEVICE_ADDR, VER_REGISTER};
    use embedded_hal_mock::eh0::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    /// DEVICE_VER_MEMS_LTS features + audio spectrum analyzer.
    const DEVICE_VER_MEMS_LTS_ASA: u8 = 0x32;

    #[test]
    fn confirm_firmware_version() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR,
            vec![VER_REGISTER],
            vec![DEVICE_VER_MEMS_LTS_ASA],
        )];
        let i2c_mock = I2cMock::new(&expectations);

        let mut pa_spl = PaSpl::new(i2c_mock);
        let version = pa_spl.get_firmware_version().unwrap();
        assert_eq!(DEVICE_VER_MEMS_LTS_ASA, version);

        let mut mock = pa_spl.destroy();
        mock.done();
    }
}
