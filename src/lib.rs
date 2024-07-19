#![cfg_attr(not(test), no_std)]

use bitfield_struct::bitfield;
use defmt::Format;
use embedded_hal::blocking::i2c;

/// PCB Artists SPL Module I2C address.
const DEVICE_ADDR: u8 = 0x48;

/// Device Registers.
const REG_CONTROL: u8 = 0x06;
pub const REG_CONTROL_DEFAULT: u8 = 0b0000_0010;

#[cfg(not(feature = "external_mic"))]
#[bitfield(u8)]
#[derive(PartialEq, Eq, Format)]
pub struct ControlRegister {
    /// Set to power down the sensor
    power_down: bool,
    /// Filter selection
    #[bits(2)]
    filter_setting: FilterSetting,
    /// Set to enable interrupt pin operation
    interrupt_enable: bool,
    /// Set to enable min/max level interrupts
    interrupt_type: bool,
    /// Padding
    #[bits(3)]
    __: u8,
}

#[cfg(feature = "external_mic")]
#[bitfield(u8)]
#[derive(PartialEq, Eq, Format)]
pub struct ControlRegister {
    /// Set to power down the sensor
    power_down: bool,
    /// Filter selection
    #[bits(2)]
    filter_setting: FilterSetting,
    /// Set to enable interrupt pin operation
    interrupt_enable: bool,
    /// Set to enable min/max level interrupts
    interrupt_type: bool,
    /// Set to enable line output (only for modules with external microphone)
    enable_line_out: bool,
    /// Padding for reserved bits
    #[bits(2)]
    __: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterSetting {
    None = 0b00,
    AWeighting = 0b01,
    CWeighting = 0b10,
}

impl FilterSetting {
    const fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => Self::None,
            0b01 => Self::AWeighting,
            _ => Self::CWeighting,
        }
    }

    const fn into_bits(self) -> u8 {
        self as _
    }
}

impl ControlRegister {
    /// Public setter for filter_setting
    pub fn set_filter(&mut self, filter_setting: FilterSetting) {
        self.set_filter_setting(filter_setting);
    }
}

const REG_RESET: u8 = 0x09;
pub const REG_RESET_DEFAULT: u8 = 0b0000_0000;

#[bitfield(u8)]
#[derive(PartialEq, Eq, Format)]
pub struct ResetRegister {
    /// Set this bit to clear interrupt signal and set INT pin to high-Z; this bit is self-clearing
    clear_interrupt: bool,
    /// Set this bit to clear the max and min dB values stored in MAX and MIN registers; t his bit is self-clearing
    clear_min_max: bool,
    /// Set this bit to clear the most recent 100 decibel values stored in history registers; this bit is self-clearing.
    clear_history: bool,
    /// Set this bit to perform a soft system reset and restore settings to defaults; this bit is self-clearing.
    // NOTE: This bit must be set to wake up the device from sleep mode.
    system_reset: bool,
    /// Padding for reserved bits
    #[bits(4)]
    __: u8,
}

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

    /// Gets the latest SPL value in decibels from the DECIBELregister.
    ///
    /// The SPL value is averaged over the last Tavg time period that is stored
    /// in the TAVG high byte register (0x07) and the TAVG low register (x08).
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

    /// Gets the CONTROL register.
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
    /// let reg_control = pa_spl.get_control_register().unwrap();
    /// ```
    pub fn get_control_register(&mut self) -> Result<ControlRegister, Error<E>> {
        let control_reg_raw = self.read_byte(REG_CONTROL)?;
        Ok(ControlRegister::from_bits(control_reg_raw))
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

    /// Gets the firmware version from the VERSION register.
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

    /// Gets the value stored in the SCRATCH register.
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

    /// Soft resets the sensor.
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
    /// let result = pa_spl.reset();
    /// ```
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        let reg_reset = ResetRegister::new().with_system_reset(true);
        self.write_byte(REG_RESET, reg_reset.into_bits())
    }

    /// Sets the CONTROL register.
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
    /// let reg_control = ControlRegister::from_bits(REG_CONTROL_DEFAULT);
    /// reg_control.set_filter_setting(FilterSetting::CWeighting);
    /// let result = pa_spl.set_control_register(reg_control);
    /// ```
    pub fn set_control_register(&mut self, reg: ControlRegister) -> Result<(), Error<E>> {
        self.write_byte(REG_CONTROL, reg.into_bits())
    }

    /// Sets the value stored in the SCRATCH register.
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
    use crate::{ControlRegister, FilterSetting, REG_CONTROL, REG_CONTROL_DEFAULT, REG_RESET};

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
            vec![REG_CONTROL_DEFAULT], // 0b0000_0010
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let reg_control = pa_spl.get_control_register().unwrap();
        let control_register_default_bits = ControlRegister::from_bits(REG_CONTROL_DEFAULT);
        assert_eq!(control_register_default_bits, reg_control);

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_set_control_register() {
        let expectations = vec![
            I2cTransaction::write_read(
                DEVICE_ADDR,
                vec![REG_CONTROL],
                vec![REG_CONTROL_DEFAULT], // 0b0000_0010
            ),
            I2cTransaction::write(
                DEVICE_ADDR,
                vec![REG_CONTROL, 0b0000_0100], // 0b0000_0100
            ),
        ];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        // Read-modify-write register.
        let mut reg_control = pa_spl.get_control_register().unwrap();
        reg_control.set_filter_setting(FilterSetting::CWeighting);
        let result = pa_spl.set_control_register(reg_control);
        assert!(result.is_ok());

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_reset() {
        let expectations = vec![I2cTransaction::write(
            DEVICE_ADDR,
            vec![REG_RESET, 0b0000_1000],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let result = pa_spl.reset();
        assert!(result.is_ok());

        let mut mock = pa_spl.destroy();
        mock.done();
    }
}
