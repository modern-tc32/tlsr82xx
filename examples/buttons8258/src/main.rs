#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;

mod platform;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::drv_platform_init();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });

    loop {
        let b1 = board.button1.is_low();

        let _ = if b1 {
            board.led_y.set_high()
        } else {
            board.led_y.set_low()
        };

        let _ = if b1 {
            board.led_w.set_high()
        } else {
            board.led_w.set_low()
        };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
