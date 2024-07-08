#![no_std]
#![no_main]

use defmt::Format;
use defmt_rtt as _; // defmt transport
use panic_probe as _; // panic handler
use stm32f3xx_hal as _; // memory layout

use pcb_artists_spl::{Error, PaSpl};
use stm32f3xx_hal::gpio::{
    gpiob::{PB8, PB9},
    Alternate, OpenDrain,
};
use stm32f3xx_hal::{i2c::I2c, pac, prelude::*};

struct State {
    pa_spl: PaSpl<I2c<pac::I2C1, (PB8<Alternate<OpenDrain, 4>>, PB9<Alternate<OpenDrain, 4>>)>>,
}

// Define a newtype wrapper for Error<E>
pub struct WrappedError<E>(pub pcb_artists_spl::Error<E>);

// Implement Format for WrappedError<E>
impl<E: Format> Format for WrappedError<E> {
    fn format(&self, f: defmt::Formatter) {
        match &self.0 {
            pcb_artists_spl::Error::I2c(err) => {
                defmt::write!(f, "I2C Error: {:?}", err);
            }
            pcb_artists_spl::Error::NoI2cInstance => {
                defmt::write!(f, "No I2C Instance available");
            } // Handle other error variants as needed
        }
    }
}

#[defmt_test::tests]
mod tests {
    use super::State;
    use defmt::{assert_eq, debug, error, unwrap};
    use pcb_artists_spl::{Error, PaSpl};
    use stm32f3xx_hal::{i2c::I2c, pac, prelude::*};

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
        let scl =
            gpiob
                .pb8
                .into_af_open_drain::<4>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrh);
        let sda =
            gpiob
                .pb9
                .into_af_open_drain::<4>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrh);
        let i2c = I2c::new(
            device_periphs.I2C1,
            (scl, sda),
            1_000_000.Hz(),
            clocks,
            &mut rcc.apb1,
        );

        let pa_spl = PaSpl::new(i2c);
        State { pa_spl }
    }

    #[test]
    fn always_passes() {
        assert!(true);
    }

    // #[test]
    // fn confirm_firmware_version(state: &mut State) {
    //     const EXPECTED: u8 = 0x32;
    //     let firmware_version = state.pa_spl.get_firmware_version().unwrap();
    //     assert_eq!(EXPECTED, firmware_version);
    // }
}
