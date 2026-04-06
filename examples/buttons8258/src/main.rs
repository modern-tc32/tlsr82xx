#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{InputPin, OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;

mod platform;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::drv_platform_init();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });

    loop {
        let b1 = InputPin::is_low(&mut board.button1).unwrap_or(false);

        let _ = board.led_y.set_state(PinState::from(b1));
        let _ = board.led_w.set_state(PinState::from(b1));
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
