#![no_std]
#![no_main]

use defmt::Format;
use defmt_rtt as _; // defmt transport.
use panic_probe as _; // Panic handler.
use stm32f3xx_hal as _; // Memory layout.

use pa_spl::{ControlRegister, Error, FilterSetting, PaSpl};
use stm32f3xx_hal::gpio::{
    gpiob::{PB6, PB7},
    Alternate, OpenDrain,
};
use stm32f3xx_hal::{i2c::I2c, pac, prelude::*};

struct State {
    pa_spl: PaSpl<I2c<pac::I2C1, (PB6<Alternate<OpenDrain, 4>>, PB7<Alternate<OpenDrain, 4>>)>>,
}

// Define a newtype wrapper for Error<E>
pub struct WrappedError<E>(pub pa_spl::Error<E>);

// Implement Format for WrappedError<E>
impl<E: Format> Format for WrappedError<E> {
    fn format(&self, f: defmt::Formatter) {
        match &self.0 {
            pa_spl::Error::I2c(err) => {
                defmt::write!(f, "I2C Error: {:?}", err);
            }
            pa_spl::Error::NoI2cInstance => {
                defmt::write!(f, "No I2C Instance available");
            }
            pa_spl::Error::BufferOverflow => {
                defmt::write!(f, "Buffer has overflowed");
            }
        }
    }
}

#[defmt_test::tests]
mod tests {
    use super::State;
    use crate::delay_ms;
    use defmt::{assert_eq, debug, error, unwrap};
    use pa_spl::{ControlRegister, Error, FilterSetting, PaSpl, REG_CONTROL_DEFAULT};
    use stm32f3xx_hal::{gpio::gpiob, i2c::I2c, pac, prelude::*};

    #[init]
    fn setup() -> State {
        // Enable and reset the cycle counter.
        let mut core_periphs = unwrap!(cortex_m::Peripherals::take());
        core_periphs.DCB.enable_trace();

        // Initialize I2C.
        let device_periphs = unwrap!(pac::Peripherals::take());
        let mut rcc = device_periphs.RCC.constrain();
        let mut flash = device_periphs.FLASH.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        let mut gpiob = device_periphs.GPIOB.split(&mut rcc.ahb);

        // Configure I2C1.
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
        let i2c = I2c::new(
            device_periphs.I2C1,
            (scl, sda),
            100.kHz().try_into().unwrap(),
            clocks,
            &mut rcc.apb1,
        );

        let pa_spl = PaSpl::new(i2c);
        State { pa_spl }
    }

    #[after_each]
    fn after_each(state: &mut State) {
        let _ = state.pa_spl.reset();
        // Short delay to allow for reset and settle.
        let ms = 3;
        delay_ms(ms);
    }

    #[test]
    fn confirm_firmware_version(state: &mut State) {
        // NOTE: The published version is 0x32 but this device returns 0x33.
        const EXPECTED: u8 = 0x33;
        let firmware_version = state.pa_spl.get_firmware_version().unwrap();
        assert_eq!(EXPECTED, firmware_version);
    }

    #[test]
    fn confirm_device_id(state: &mut State) {
        const EXPECTED: u32 = 1867099226; // TODO Device ID is not published - need to verify somehow, #5.
        let device_id = state.pa_spl.get_device_id().unwrap();
        assert_eq!(EXPECTED, device_id);
    }

    #[test]
    fn confirm_get_avg_time(state: &mut State) {
        const EXPECTED: u16 = 1000;
        let avg_time = state.pa_spl.get_avg_time().unwrap();
        assert_eq!(EXPECTED, avg_time);
    }

    #[test]
    fn confirm_get_control_register(state: &mut State) {
        const EXPECTED: ControlRegister = ControlRegister::from_bits(REG_CONTROL_DEFAULT);
        let reg_control = state.pa_spl.get_control_register().unwrap();
        assert_eq!(EXPECTED, reg_control);
    }

    #[test]
    fn confirm_get_max_decibel(state: &mut State) {
        // The value returned is a sensed value, so this only tests that a valid
        // result is returned.
        let result = state.pa_spl.get_max_decibel();
        assert!(result.is_ok());
    }

    #[test]
    fn confirm_get_min_decibel(state: &mut State) {
        // The value returned is a sensed value, so this only tests that a valid
        // result is returned.
        let result = state.pa_spl.get_min_decibel();
        assert!(result.is_ok());
    }

    #[test]
    fn confirm_read_latest_decibel(state: &mut State) {
        // The value returned is a sensed value, so this only tests that a valid
        // result is returned.
        let result = state.pa_spl.get_latest_decibel();
        assert!(result.is_ok());
    }

    #[test]
    fn confirm_reset(state: &mut State) {
        let result = state.pa_spl.reset();
        assert!(result.is_ok());

        // Short delay to allow for reset and settle.
        let ms = 3;
        delay_ms(ms);

        // Confirm that the settings have been reset to the default.
        const EXPECTED: ControlRegister = ControlRegister::from_bits(REG_CONTROL_DEFAULT);
        let reg_control = state.pa_spl.get_control_register().unwrap();
        assert_eq!(EXPECTED, reg_control);
    }

    #[test]
    fn confirm_rw_scratch(state: &mut State) {
        const EXPECTED_VAL: u8 = 0x99;
        let write_result = state.pa_spl.set_scratch(EXPECTED_VAL);
        assert!(write_result.is_ok());

        let val = state.pa_spl.get_scratch().unwrap();
        assert_eq!(EXPECTED_VAL, val);
    }

    #[test]
    fn confirm_set_avg_time(state: &mut State) {
        let new_avg_time_ms: u16 = 125;
        let result = state.pa_spl.set_avg_time(new_avg_time_ms);
        assert!(result.is_ok());

        const EXPECTED: u16 = 125;
        let avg_time = state.pa_spl.get_avg_time().unwrap();
        assert_eq!(EXPECTED, avg_time);
    }

    #[test]
    fn confirm_set_control_register(state: &mut State) {
        const EXPECTED_DEFAULT: ControlRegister = ControlRegister::from_bits(REG_CONTROL_DEFAULT);
        let mut reg_control = state.pa_spl.get_control_register().unwrap();
        assert_eq!(EXPECTED_DEFAULT, reg_control);

        reg_control.set_filter(FilterSetting::CWeighting);
        let result = state.pa_spl.set_control_register(reg_control);
        assert!(result.is_ok());

        const EXPECTED_SET: ControlRegister = ControlRegister::from_bits(0b0000_0100);
        let reg_control_set = state.pa_spl.get_control_register().unwrap();
        assert_eq!(EXPECTED_SET, reg_control_set);
    }

    #[test]
    fn sanity_check() {
        assert!(true);
    }
}

/// Busy-wait loop to create a delay.
fn delay_ms(ms: u32) {
    let cycles_per_ms = 8_000; // Assuming an 8 MHz clock (stm32f4-hal default for 303xc)
    for _ in 0..(ms * cycles_per_ms) {
        cortex_m::asm::nop(); // No operation (NOP) to prevent optimization
    }
}
