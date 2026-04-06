use tlsr82xx_hal::analog::Pull;
use tlsr82xx_hal::gpio::{
    DriveStrength, GpioExt, Input, Level, Output, PinFunction, PA7, PB4, PB5, Pins,
};
use tlsr82xx_hal::pac;
use tlsr82xx_hal::uart::{self, Pins as UartPins};

pub struct Board {
    pub led_y: PB4<Output>,
    pub led_w: PB5<Output>,
    pub button1: PA7<Input>,
}

impl Board {
    pub fn from_pins(pins: Pins) -> Self {
        let mut led_y = pins.pb4.into_output_with_state(Level::High);
        let mut led_w = pins.pb5.into_output_with_state(Level::Low);
        let mut button1 = pins.pa7.into_input();

        led_y.set_drive_strength(DriveStrength::Strong);
        led_w.set_drive_strength(DriveStrength::Strong);
        button1.set_pull_resistor(Pull::PullUp1M);

        Self {
            led_y,
            led_w,
            button1,
        }
    }

    pub fn from_peripherals(peripherals: pac::Peripherals) -> Self {
        Self::from_pins(peripherals.gpio.split())
    }

    pub fn button1_pressed(&self) -> bool {
        self.button1.is_low()
    }
}

pub fn configure_rgb_pins(pins: &mut Pins) {
    pins.pc2.set_function(PinFunction::Pwm0);
    pins.pc3.set_function(PinFunction::Pwm1);
    pins.pc4.set_function(PinFunction::Pwm2);
}

pub fn configure_uart_pins() {
    uart::apply_pins(UartPins::PB1_PA0);
}
