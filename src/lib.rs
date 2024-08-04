#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

use bitfield_struct::bitfield;
use defmt::Format;
use embedded_hal::blocking::i2c;

/// PCB Artists SPL Module I2C default address.
const DEVICE_ADDR_DEFAULT: u8 = 0x48;

/// CONTROL register address.
const REG_CONTROL: u8 = 0x06;
/// CONTROL register default value.
pub const REG_CONTROL_DEFAULT: u8 = 0x02;

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
    /// No filter
    None = 0b00,
    /// A-weighting
    AWeighting = 0b01,
    /// C-weighting
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
    /// Sets the filter setting
    ///
    /// Sets the filter setting bits in ControlRegister for a filter defined in FilterSetting.
    ///
    pub fn set_filter(&mut self, filter_setting: FilterSetting) {
        self.set_filter_setting(filter_setting);
    }
}

/// RESET register address.
const REG_RESET: u8 = 0x09;
/// RESET register default value.
pub const REG_RESET_DEFAULT: u8 = 0x00;

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
    /// NOTE: This bit must be set to wake up the device from sleep mode.
    system_reset: bool,
    /// Padding for reserved bits
    #[bits(4)]
    __: u8,
}

/// VESION register address.
const REG_VERSION: u8 = 0x00;
/// DECIBEL register address.
const REG_DECIBEL: u8 = 0x0a;
/// Device ID registers, ID3, ID2, ID1, ID0
const REGS_DEVICE_ID: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
/// MAX register.
const REG_MAX: u8 = 0x0c;
/// MIN register.
const REG_MIN: u8 = 0x0d;
/// SCRATCH register address.
const REG_SCRATCH: u8 = 0x05;
/// TAVG register high byte address.
const REG_TAVG_HIGH: u8 = 0x07;
/// Default value for averaging time in ms.
pub const REG_AVERAGING_TIME_DEFAULT_MS: u16 = 1000;

/// GAIN register.
#[cfg(feature = "external_mic")]
const REG_GAIN: u8 = 0x0f;

/// A PA SPL Module on the I2C bus `I2C`.
pub struct PaSpl<I2C>
where
    I2C: i2c::Read + i2c::Write + i2c::WriteRead,
{
    i2c: Option<I2C>,
    device_addr: u8,
}

/// A driver error.
#[derive(Debug, PartialEq)]
pub enum Error<E> {
    /// I2C bus error.
    I2c(E),
    /// No I2C instance available.
    NoI2cInstance,
    /// Buffer overflow.
    BufferOverflow,
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
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c: Some(i2c),
            device_addr: DEVICE_ADDR_DEFAULT,
        }
    }

    /// Sets a new I2C device address.
    ///
    /// The published device address is the default but the vendor's website
    /// states that it is possible to order a device with a custom address.
    ///
    pub fn set_device_addr(&mut self, addr: u8) {
        self.device_addr = addr;
    }

    /// Gets the 16-bit averaging time in ms from registers TAVG high and TAVG low (0x07 and 0x08).
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    pub fn get_avg_time(&mut self) -> Result<u16, Error<E>> {
        let mut buffer: [u8; 2] = [0; 2];
        self.read_bytes(REG_TAVG_HIGH, &mut buffer)?;

        // Combine the bytes into a u16.
        let avg_time_ms = ((buffer[0] as u16) << 8) | (buffer[1] as u16);

        Ok(avg_time_ms)
    }

    /// Gets the CONTROL register.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
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
    pub fn get_device_id(&mut self) -> Result<u32, Error<E>> {
        let mut buffer: [u8; 4] = [0; 4];
        self.read_bytes(REGS_DEVICE_ID[0], &mut buffer)?;

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
    pub fn get_firmware_version(&mut self) -> Result<u8, Error<E>> {
        self.read_byte(REG_VERSION)
    }

    /// Gets the gain value in 0.5 decibel steps from the GAIN register.
    ///
    /// This value only needs to be modified if you are using your own
    /// microphone. The default value will work with the default microphone
    /// supplied with the module.
    ///
    /// Acceptable values are 0 to 95 to set the gain in 0.5 dB steps (+0.0 dB
    /// to +47.5 dB).
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    #[cfg(feature = "external_mic")]
    pub fn get_gain(&mut self) -> Result<u8, Error<E>> {
        self.read_byte(REG_GAIN)
    }

    /// Gets the latest SPL value in decibels from the DECIBEL register.
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
    pub fn get_latest_decibel(&mut self) -> Result<u8, Error<E>> {
        self.read_byte(REG_DECIBEL)
    }

    /// Gets the max SPL value in decibels from the MAX register.
    ///
    /// Maximum value of decibel reading captured since power-up or manual reset of MIN/MAX registers.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    pub fn get_max_decibel(&mut self) -> Result<u8, Error<E>> {
        self.read_byte(REG_MAX)
    }

    /// Gets the min SPL value in decibels from the MIN register.
    ///
    /// Minimum value of decibel reading captured since power-up or manual reset of MIN/MAX registers.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    pub fn get_min_decibel(&mut self) -> Result<u8, Error<E>> {
        self.read_byte(REG_MIN)
    }

    /// Gets the value stored in the SCRATCH register.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    pub fn get_scratch(&mut self) -> Result<u8, Error<E>> {
        self.read_byte(REG_SCRATCH)
    }

    /// Soft resets the sensor.
    ///
    /// The sensor is soft reset by setting the System Reset bit in the RESET register.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        let reg_reset = ResetRegister::new().with_system_reset(true);
        self.write_byte(REG_RESET, reg_reset.into_bits())
    }

    /// Sets the average time in ms for calculating SPL.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    pub fn set_avg_time(&mut self, ms: u16) -> Result<(), Error<E>> {
        // Convert the average time in ms to high and low bytes.
        let tavg_high_byte: u8 = (ms >> 8) as u8;
        let tavg_low_byte: u8 = (ms & 0xFF) as u8;
        let buffer = [tavg_high_byte, tavg_low_byte];

        self.write_two_bytes(REG_TAVG_HIGH, &buffer)
    }

    /// Sets the CONTROL register.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    pub fn set_control_register(&mut self, reg: ControlRegister) -> Result<(), Error<E>> {
        self.write_byte(REG_CONTROL, reg.into_bits())
    }

    /// Sets the gain in the GAIN register.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    /// ```
    #[cfg(feature = "external_mic")]
    pub fn set_gain(&mut self, value: u8) -> Result<(), Error<E>> {
        self.write_byte(REG_GAIN, value)
    }

    /// Sets the value stored in the SCRATCH register.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoI2cInstance`] if the I2C instance is empty.
    ///
    /// Returns [`Error::I2c`] if I2C returns an error.
    ///
    pub fn set_scratch(&mut self, value: u8) -> Result<(), Error<E>> {
        self.write_byte(REG_SCRATCH, value)
    }

    /// Destroys this driver and releases the I2C bus.
    ///
    pub fn destroy(&mut self) -> I2C {
        self.i2c
            .take()
            .expect("I2C instance has already been taken")
    }

    /// Reads a single byte from an I2C register of the device.
    /// 
    fn read_byte(&mut self, reg: u8) -> Result<u8, Error<E>> {
        let mut buffer = [0; 1];
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write_read(self.device_addr, &[reg], &mut buffer)
            .map_err(Error::I2c)?;
        Ok(buffer[0])
    }

    /// Read multiple bytes from a starting register.
    /// 
    fn read_bytes(&mut self, start_reg: u8, buffer: &mut [u8]) -> Result<(), Error<E>> {
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write_read(self.device_addr, &[start_reg], buffer)
            .map_err(Error::I2c)?;
        Ok(())
    }

    /// Writes a single byte to an I2C register of the device.
    /// 
    fn write_byte(&mut self, reg: u8, value: u8) -> Result<(), Error<E>> {
        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write(self.device_addr, &[reg, value])
            .map_err(Error::I2c)
    }

    /// Writes two bytes from a starting register.
    /// 
    fn write_two_bytes(&mut self, reg: u8, buffer: &[u8]) -> Result<(), Error<E>> {
        if buffer.len() > 2 {
            return Err(Error::BufferOverflow);
        }

        self.i2c
            .as_mut()
            .ok_or(Error::NoI2cInstance)?
            .write(self.device_addr, &[reg, buffer[0], buffer[1]])
            .map_err(Error::I2c)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ControlRegister, FilterSetting, REG_AVERAGING_TIME_DEFAULT_MS, REG_CONTROL,
        REG_CONTROL_DEFAULT, REG_RESET, REG_TAVG_HIGH,
    };

    use super::*;
    use embedded_hal_mock::eh0::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    /// DEVICE_VER_MEMS_LTS: Published version for base features + audio spectrum analyzer.
    const DEVICE_VER_MEMS_LTS_ASA: u8 = 0x32;
    /// TAVG register high byte default value.
    const REG_TAVG_HIGH_DEFAULT_BYTE: u8 = 0x03;
    /// TAVG register low byte default value.
    const REG_TAVG_LOW_DEFAULT_BYTE: u8 = 0xE8;

    #[test]
    fn confirm_set_device_addr() {
        let expectations = vec![];
        let i2c_mock = I2cMock::new(&expectations);

        let mut pa_spl = PaSpl::new(i2c_mock);
        let new_device_addr: u8 = 0x99;
        pa_spl.set_device_addr(new_device_addr);
        assert_eq!(new_device_addr, pa_spl.device_addr);

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_device_id() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR_DEFAULT,
            vec![REGS_DEVICE_ID[0]],
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
            DEVICE_ADDR_DEFAULT,
            vec![REG_VERSION],
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
    fn confirm_get_avg_time() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR_DEFAULT,
            vec![REG_TAVG_HIGH],
            vec![REG_TAVG_HIGH_DEFAULT_BYTE, REG_TAVG_LOW_DEFAULT_BYTE],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let averaging_time_ms = pa_spl.get_avg_time().unwrap();
        assert_eq!(REG_AVERAGING_TIME_DEFAULT_MS, averaging_time_ms);

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_get_control_register() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR_DEFAULT,
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

    #[cfg(feature = "external_mic")]
    #[test]
    fn confirm_get_gain() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR_DEFAULT,
            vec![REG_GAIN],
            vec![18],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let expected_gain: u8 = 18;
        let gain_val = pa_spl.get_gain().unwrap();
        assert_eq!(expected_gain, gain_val);

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_get_latest_decibel() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR_DEFAULT,
            vec![REG_DECIBEL],
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
    fn confirm_get_max_decibel() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR_DEFAULT,
            vec![REG_MAX],
            vec![0x12],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let result = pa_spl.get_max_decibel();
        assert!(result.is_ok());

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_get_min_decibel() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR_DEFAULT,
            vec![REG_MIN],
            vec![0x12],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let result = pa_spl.get_min_decibel();
        assert!(result.is_ok());

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_get_scratch() {
        let expectations = vec![I2cTransaction::write_read(
            DEVICE_ADDR_DEFAULT,
            vec![REG_SCRATCH],
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
    fn confirm_reset() {
        let expectations = vec![I2cTransaction::write(
            DEVICE_ADDR_DEFAULT,
            vec![REG_RESET, 0b0000_1000],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let result = pa_spl.reset();
        assert!(result.is_ok());

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_set_avg_time() {
        let new_avg_time_ms: u16 = 125;
        let tavg_high_expected_byte: u8 = 0x00;
        let tavg_low_expected_byte: u8 = 0x7D;
        let expectations = vec![I2cTransaction::write(
            DEVICE_ADDR_DEFAULT,
            vec![
                REG_TAVG_HIGH,
                tavg_high_expected_byte,
                tavg_low_expected_byte,
            ],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let result = pa_spl.set_avg_time(new_avg_time_ms);
        assert!(result.is_ok());

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_set_control_register() {
        let expectations = vec![
            I2cTransaction::write_read(
                DEVICE_ADDR_DEFAULT,
                vec![REG_CONTROL],
                vec![REG_CONTROL_DEFAULT], // 0b0000_0010
            ),
            I2cTransaction::write(
                DEVICE_ADDR_DEFAULT,
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

    #[cfg(feature = "external_mic")]
    #[test]
    fn confirm_set_gain() {
        let new_gain_val: u8 = 43;
        let expectations = vec![I2cTransaction::write(
            DEVICE_ADDR_DEFAULT,
            vec![REG_GAIN, new_gain_val],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let result = pa_spl.set_gain(new_gain_val);
        assert!(result.is_ok());

        let mut mock = pa_spl.destroy();
        mock.done();
    }

    #[test]
    fn confirm_set_scratch() {
        let scratch_write_val: u8 = 0x99;
        let expectations = vec![I2cTransaction::write(
            DEVICE_ADDR_DEFAULT,
            vec![REG_SCRATCH, scratch_write_val],
        )];
        let i2c_mock = I2cMock::new(&expectations);
        let mut pa_spl = PaSpl::new(i2c_mock);

        let result = pa_spl.set_scratch(scratch_write_val);
        assert!(result.is_ok());

        let mut mock = pa_spl.destroy();
        mock.done();
    }
}
