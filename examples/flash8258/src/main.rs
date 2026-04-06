#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::flash::Flash;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;

mod platform;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    let flash = Flash::new();
    let mid = flash.read_jedec_id();
    let mut uid = [0u8; 16];
    let uid_ok = flash.read_uid_default(&mut uid).is_ok();
    let calib = flash.read_vdd_f_calibration_value();
    let mut tick = timer::clock_time();
    let mut frame = 0u8;

    loop {
        if timer::clock_time_exceed_us(tick, 400_000) {
            tick = timer::clock_time();
            frame = (frame + 1) % 3;

            let (yellow, white) = match frame {
                0 => (mid.is_zbit(), !mid.is_zbit()),
                1 => (uid_ok, (uid[0] & 1) != 0),
                _ => (calib != 0xff, (calib & 1) != 0),
            };

            drive_pin(&mut board.led_y, yellow);
            drive_pin(&mut board.led_w, white);
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
