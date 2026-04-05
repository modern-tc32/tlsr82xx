use tlsr82xx_hal::analog::Pull;
use tlsr82xx_hal::gpio::{DriveStrength, GpioExt, Level, PA7, PB4, PB5, PD2};
use tlsr82xx_hal::pac;

use crate::platform;

pub struct Board {
    pub led_y: PB4<tlsr82xx_hal::gpio::Output>,
    pub led_w: PB5<tlsr82xx_hal::gpio::Output>,
    pub button1: PA7<tlsr82xx_hal::gpio::Input>,
    pub button2: PD2<tlsr82xx_hal::gpio::Input>,
}

pub fn init() -> Board {
    unsafe {
        let _ = platform::drv_platform_init();
    }

    let peripherals = unsafe { pac::Peripherals::steal() };
    let pins = peripherals.gpio.split();
    let mut led_y = pins.pb4.into_output_with_state(Level::High);
    let mut led_w = pins.pb5.into_output_with_state(Level::Low);
    let mut button1 = pins.pa7.into_input();
    let mut button2 = pins.pd2.into_input();

    led_y.set_drive_strength(DriveStrength::Strong);
    led_w.set_drive_strength(DriveStrength::Strong);
    button1.set_pull_resistor(Pull::PullUp1M);
    button2.set_pull_resistor(Pull::PullUp10K);

    Board {
        led_y,
        led_w,
        button1,
        button2,
    }
}

impl Board {
    pub fn button1_pressed(&self) -> bool {
        self.button1.is_low()
    }

    pub fn button2_pressed(&self) -> bool {
        self.button2.is_low()
    }
}
