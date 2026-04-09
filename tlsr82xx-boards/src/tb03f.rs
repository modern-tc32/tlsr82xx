use embedded_hal::digital::InputPin;
use tlsr82xx_hal::analog::Pull;
use tlsr82xx_hal::gpio::{
    self, DriveStrength, GpioExt, Input, Level, Output, PinFunction, PinmuxError, Pins, RawPin,
    PA7, PB2, PB3, PB4, PB5,
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

    pub fn button1_pressed(&mut self) -> bool {
        InputPin::is_low(&mut self.button1).unwrap_or(false)
    }
}

pub fn configure_rgb_pins(pins: &mut Pins) {
    let _ = pins.pc2.set_function(PinFunction::Pwm0);
    let _ = pins.pc3.set_function(PinFunction::Pwm1);
    let _ = pins.pc4.set_function(PinFunction::Pwm2);
}

pub fn configure_uart_pins() {
    uart::apply_pins(UartPins::PB1_PA0);
}

pub fn configure_radio_fe_pins() -> Result<(), PinmuxError> {
    const PIN_PB2_RAW: RawPin = PB2::<Input>::raw_pin();
    const PIN_PB3_RAW: RawPin = PB3::<Input>::raw_pin();
    gpio::set_function_for_raw_pin(PIN_PB2_RAW, PinFunction::RxCyc2Lna)?;
    gpio::set_function_for_raw_pin(PIN_PB3_RAW, PinFunction::TxCyc2Pa)?;
    Ok(())
}
