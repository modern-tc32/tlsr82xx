#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;

mod platform;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    let mut tick = timer::clock_time();
    let mut led_y_on = false;

    loop {
        if timer::clock_time_exceed_us(tick, 500_000) {
            tick = timer::clock_time();
            led_y_on = !led_y_on;
            if led_y_on {
                drive_pin(&mut board.led_y, true);
                drive_pin(&mut board.led_w, false);
            } else {
                drive_pin(&mut board.led_y, false);
                drive_pin(&mut board.led_w, true);
            }
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
