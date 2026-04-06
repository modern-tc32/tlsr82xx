#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::adc::{Adc, AdcGpioPin};
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;

mod platform;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    let adc = Adc::new();
    adc.init_gpio_input(AdcGpioPin::Pb3);
    let mut tick = timer::clock_time();
    let mut frame = 0u8;

    loop {
        if timer::clock_time_exceed_us(tick, 400_000) {
            tick = timer::clock_time();
            frame = (frame + 1) % 3;

            let sample = adc.sample_current_config_with_fluctuation();
            let calib = adc.gpio_calibration_vref_mv() as u32;
            let bits = match frame {
                0 => sample.millivolts,
                1 => sample.fluctuation_mv,
                _ => calib,
            };

            drive_pin(&mut board.led_y, (bits & 0x01) != 0);
            drive_pin(&mut board.led_w, (bits & 0x02) != 0);
        }
    }
}

fn drive_pin<P: OutputPin>(pin: &mut P, high: bool) {
    let _ = pin.set_state(PinState::from(high));
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
