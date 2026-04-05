#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::OutputPin;
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;

mod platform;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        let _ = platform::drv_platform_init();
    }

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    let mut tick = timer::clock_time();
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
    let _ = if high { pin.set_high() } else { pin.set_low() };
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
