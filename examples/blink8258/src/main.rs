#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::OutputPin;

mod board;
mod platform;
mod time;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let mut board = board::init();
    let mut tick = time::clock_time();
    let mut led_y_on = true;

    loop {
        if board.button1_pressed() {
            drive_pin(&mut board.led_y, true);
            drive_pin(&mut board.led_w, false);
            continue;
        }

        if board.button2_pressed() {
            drive_pin(&mut board.led_y, false);
            drive_pin(&mut board.led_w, true);
            continue;
        }

        if time::clock_time_exceed(tick, 500_000) {
            tick = time::clock_time();
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
    let _ = if high { pin.set_high() } else { pin.set_low() };
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
