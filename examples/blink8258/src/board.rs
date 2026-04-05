use tlsr82xx_hal::gpio::{DriveStrength, GpioExt, Level, PB4, PB5};
use tlsr82xx_hal::pac;

use crate::platform;

pub struct Board {
    pub led_y: PB4<tlsr82xx_hal::gpio::Output>,
    pub led_w: PB5<tlsr82xx_hal::gpio::Output>,
}

pub fn init() -> Board {
    unsafe {
        let _ = platform::drv_platform_init();
    }

    let peripherals = unsafe { pac::Peripherals::steal() };
    let pins = peripherals.gpio.split();
    let mut led_y = pins.pb4.into_output_with_state(Level::High);
    let mut led_w = pins.pb5.into_output_with_state(Level::Low);

    led_y.set_drive_strength(DriveStrength::Strong);
    led_w.set_drive_strength(DriveStrength::Strong);

    Board { led_y, led_w }
}
