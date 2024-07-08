#![no_std]
#![no_main]

use defmt_rtt as _; // defmt transport
use panic_probe as _; // panic handler
use stm32f3xx_hal as _; // memory layout

use pcb_artists_spl::PaSpl;
use stm32f3xx_hal::{i2c::I2c, pac, prelude::*};

// struct State {
//     pa_spl: PaSpl<I2c<pac::I2C1>>,
// }

#[defmt_test::tests]
mod tests {
    use defmt::{assert_eq, unwrap};
    // use stm32f3xx_hal::{i2c::I2c, pac, prelude::*};

    // use super::State;

    // #[init]
    // fn setup() -> State {
    //     // Enable and reset the cycle counter.
    //     let mut core_periphs = unwrap!(cortex_m::Peripherals::take());
    //     core_periphs.DCB.enable_trace();
    //     unsafe { core_periphs.DWT.cyccnt.write(0) }
    //     core_periphs.DWT.enable_cycle_counter();
    //     defmt::timestamp!("{=u32:µs}", cortex_m::peripheral::DWT::get_cycle_count());

    //     // Initialize I2C.
    //     let device_periphs = unwrap!(pac::Peripherals::take());
    //     let mut rcc = device_periphs.RCC.constrain();
    //     let mut flash = device_periphs.FLASH.constrain();
    //     let clocks = rcc.cfgr.freeze(&mut flash.acr);

    //     let mut gpioa = device_periphs.GPIOA.split(&mut rcc.ahb);
    //     let scl = gpioa.pa9.into_af4(&mut gpioa.moder, &mut gpioa.afrh);
    //     let sda = gpioa.pa10.into_af4(&mut gpioa.moder, &mut gpioa.afrh);
    //     let i2c = I2c::new(
    //         device_periphs.I2C1,
    //         (scl, sda),
    //         100_000.Hz(),
    //         clocks,
    //         &mut rcc.apb1,
    //     );

    //     let pa_spl = PaSpl::new(i2c);
    //     State { pa_spl }
    // }

    #[test]
    fn always_passes() {
        assert!(true);
    }

    // #[test]
    // fn confirm_firmware_version(state: &mut State) {
    //     const EXPECTED: [u8; 1] = [0x32];
    //     let firmware_version = state.pa_spl.read_firmware_version().unwrap();
    //     assert_eq!(EXPECTED, firmware_version);
    // }
}
