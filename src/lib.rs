#![cfg_attr(not(test), no_std)]

use embedded_hal::blocking::i2c;

/// PCB Artists SPL Module I2C address.
const DEVICE_ADDR: u8 = 0x48;

/// Device Registers.
const REG_CONTROL: u8 = 0x06;
const REG_CONTROL_DEFAULT: u8 = 0x02;

const VER_REGISTER: u8 = 0x00;
const DECIBEL_REGISTER: u8 = 0x0A;
const DEVICE_ID_REGISTERS: [u8; 4] = [0x01, 0x02, 0x03, 0x04]; // ID3, ID2, ID1, ID0
const SCRATCH_REGISTER: u8 = 0x05;

/// A PA SPL Module on the I2C bus `I2C`.
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
    /// Initializes the PCB Artists SPL Module driver.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// # Examples
    /// ```ignore
    /// // Configure clocks.
    /// let device_periphs = unwrap!(pac::Peripherals::take());
    /// let mut rcc = device_periphs.RCC.constrain();
    /// let mut flash = device_periphs.FLASH.constrain();
    /// let clocks = rcc.cfgr.freeze(&mut flash.acr);
    ///
    /// // Get the GRIO port for the pins needed.
    /// let mut gpiob = device_periphs.GPIOB.split(&mut rcc.ahb);
    ///
    /// // Configure pins and create an instance of I2C1.
    /// let mut scl =
    ///     gpiob
    ///         .pb6
    ///         .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    /// let mut sda =
    ///     gpiob
    ///         .pb7
    ///         .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    /// scl.internal_pull_up(&mut gpiob.pupdr, true);
    /// sda.internal_pull_up(&mut gpiob.pupdr, true);
    /// let i2c = I2c::new(
    ///     device_periphs.I2C1,
    ///     (scl, sda),
    ///     100.kHz().try_into().unwrap(),
    ///     clocks,
    ///     &mut rcc.apb1,
    /// );
    ///
    /// let pa_spl = PaSpl::new(i2c);
    /// ```
    pub fn new(i2c: I2C) -> Self {
        Self { i2c: Some(i2c) }
    }

    /// Gets the latest sound intensity value in decibels averaged over the last Tavg time period.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let latest_decibel_val = pa_spl.get_latest_decibel().unwrap();
    /// ```
    pub fn get_latest_decibel(&mut self) -> Result<u8, Error<E>> {
        self.read_byte(DECIBEL_REGISTER)
    }

    /// Gets the 32-bit device ID from registers ID3-ID0.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let device_id = pa_spl.get_device_id().unwrap();
    /// ```
    ///
    pub fn get_device_id(&mut self) -> Result<u32, Error<E>> {
        let mut buffer: [u8; 4] = [0; 4];
        self.read_bytes(DEVICE_ID_REGISTERS[0], &mut buffer)?;

        // Combine the bytes into a u32.
        let device_id: u32 = ((buffer[0] as u32) << 24)
            | ((buffer[1] as u32) << 16)
            | ((buffer[2] as u32) << 8)
            | (buffer[3] as u32);

        Ok(device_id)
    }

    /// Gets the firmware version.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let firmware_version = pa_spl.get_firmware_version().unwrap();
    /// ```
    pub fn get_firmware_version(&mut self) -> Result<u8, Error<E>> {
        self.read_byte(VER_REGISTER)
    }

    /// Gets the value stored in the scratch register.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let val = pa_spl.get_scratch().unwrap();
    /// ```
    pub fn get_scratch(&mut self) -> Result<u8, Error<E>> {
        self.read_byte(SCRATCH_REGISTER)
    }

    /// Sets the value stored in the scratch register.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let scratch_val = 0x99;
    /// let result = pa_spl.set_scratch(scratch_val);
    /// ```
    pub fn set_scratch(&mut self, value: u8) -> Result<(), Error<E>> {
        self.write_byte(SCRATCH_REGISTER, value)
    }

    /// Destroys this driver and releases the I2C bus.
    pub fn destroy(&mut self) -> I2C {
        self.i2c
            .take()
            .expect("I2C instance has already been taken")
    }

    /// Writes a single byte to an I2C register of the device.
    fn write_byte(&mut self, reg: u8, value: u8) -> Result<(), Error<E>> {
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write(DEVICE_ADDR, &[reg, value])
            .map_err(Error::I2c)
    }

    /// Reads a single byte from an I2C register of the device.
    fn read_byte(&mut self, reg: u8) -> Result<u8, Error<E>> {
        let mut buffer = [0; 1];
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write_read(DEVICE_ADDR, &[reg], &mut buffer)
            .map_err(Error::I2c)?;
        Ok(buffer[0])
    }

    /// Read multiple bytes from a starting register.
    fn read_bytes(&mut self, start_reg: u8, buffer: &mut [u8]) -> Result<(), Error<E>> {
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write_read(DEVICE_ADDR, &[start_reg], buffer)
            .map_err(Error::I2c)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{REG_CONTROL, REG_CONTROL_DEFAULT};

    use super::{
        PaSpl, DECIBEL_REGISTER, DEVICE_ADDR, DEVICE_ID_REGISTERS, SCRATCH_REGISTER, VER_REGISTER,
    };
    use embedded_hal_mock::eh0::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    /// DEVICE_VER_MEMS_LTS: Published version for base features + audio spectrum analyzer.
    const DEVICE_VER_MEMS_LTS_ASA: u8 = 0x32;

    #[test]
    fn confirm_get_latest_decibel() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR,
            vec![DECIBEL_REGISTER],
            vec![0x12],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let latest_decibel_val = pa_spl.get_latest_decibel().unwrap();
        assert_eq!(0x12, latest_decibel_val);

        let mut mock = pa_spl.destroy();
        mock.done();
    }

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

    #[test]
    fn confirm_get_control_register() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR,
            vec![REG_CONTROL],
            vec![REG_CONTROL_DEFAULT], // 0b00000010
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let reg_control = pa_spl.get_firmware_version().unwrap();
        assert_eq!(REG_CONTROL_DEFAULT, reg_control);

        let mut mock = pa_spl.destroy();
        mock.done();
    }
}
