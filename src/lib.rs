#![cfg_attr(not(test), no_std)]

use embedded_hal::blocking::i2c;

/// PCB Artists SPL Module I2C address.
const DEVICE_ADDR: u8 = 0x48;

/// Device Registers.
const VER_REGISTER: u8 = 0x00;
const DEVICE_ID_REGISTERS: [u8; 4] = [0x01, 0x02, 0x03, 0x04]; // ID3, ID2, ID1, ID0
const SCRATCH_REGISTER: u8 = 0x05;

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

    pub fn get_scratch(&mut self) -> Result<u8, Error<E>> {
        let mut buffer = [0; 1];
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write_read(DEVICE_ADDR, &[SCRATCH_REGISTER], &mut buffer)
            .map_err(Error::I2c)?;

        Ok(buffer[0])
    }

    pub fn set_scratch(&mut self, value: u8) -> Result<(), Error<E>> {
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write(DEVICE_ADDR, &[SCRATCH_REGISTER, value])
            .map_err(Error::I2c)
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
    use super::{PaSpl, DEVICE_ADDR, DEVICE_ID_REGISTERS, SCRATCH_REGISTER, VER_REGISTER};
    use embedded_hal_mock::eh0::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    /// DEVICE_VER_MEMS_LTS: Published version for base features + audio spectrum analyzer.
    const DEVICE_VER_MEMS_LTS_ASA: u8 = 0x32;

    #[test]
    fn confirm_device_id() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR,
            vec![DEVICE_ID_REGISTERS[0]],
            vec![0x01, 0x02, 0x03, 0x04],
        )];
        let i2c_mock = I2cMock::new(&expectations);

        let mut pa_spl = PaSpl::new(i2c_mock);
        let device_id = pa_spl.get_device_id().unwrap();
        assert_eq!(0x01020304, device_id);

        let mut mock = pa_spl.destroy();
        mock.done();
    }

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

    #[test]
    fn confirm_get_scratch() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR,
            vec![SCRATCH_REGISTER],
            vec![0x99],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let scratch_write_val: u8 = 0x99;
        let scratch_read_val = pa_spl.get_scratch().unwrap();
        assert_eq!(scratch_write_val, scratch_read_val);

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_set_scratch() {
        let scratch_write_val: u8 = 0x99;
        let expectations = vec![I2cTransaction::write(
            DEVICE_ADDR,
            vec![SCRATCH_REGISTER, scratch_write_val],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let result = pa_spl.set_scratch(scratch_write_val);
        assert!(result.is_ok());

        let mut mock = pa_spl.destroy();
        mock.done();
    }
}
