#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use embedded_hal::i2c::I2c as _;
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::gpio::GpioExt;
use tlsr82xx_hal::i2c::I2c;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;

mod platform;

const SCAN_START: u8 = 0x08;
const SCAN_END: u8 = 0x77;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    let peripherals = unsafe { pac::Peripherals::steal() };
    let mut pins = peripherals.gpio.split();
    let mut i2c = I2c::with_pins((&mut pins.pc0, &mut pins.pc1), 100_000);
    let mut board = Board::from_pins(pins);

    let mut tick = timer::clock_time();

    loop {
        if timer::clock_time_exceed_us(tick, 500_000) {
            tick = timer::clock_time();

            let (found_count, first_address) = scan_bus(&mut i2c);

            drive_pin(&mut board.led_y, found_count != 0);
            drive_pin(&mut board.led_w, (first_address & 1) != 0);
        }
    }
}

fn scan_bus(i2c: &mut I2c) -> (u8, u8) {
    let mut found_count = 0u8;
    let mut first_address = 0u8;

    for address in SCAN_START..=SCAN_END {
        if i2c.write(address, &[]).is_ok() {
            if found_count == 0 {
                first_address = address;
            }
            found_count = found_count.saturating_add(1);
        }
    }

    (found_count, first_address)
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
