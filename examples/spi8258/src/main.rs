#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use embedded_hal::spi::SpiBus;
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::spi::{Config, Spi, SpiPinGroup};
use tlsr82xx_hal::timer;

mod platform;

const SPI_TRANSFER_PERIOD_US: u32 = 500_000;
const INITIAL_TEST_PATTERN: u8 = 0x3c;
const LED_Y_MASK: u8 = 1 << 0;
const LED_W_MASK: u8 = 1 << 1;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    let mut spi = Spi::new(Config::new(
        SpiPinGroup::A2A3A4D6,
        1_000_000,
        embedded_hal::spi::MODE_0,
    ));
    let mut tick = timer::clock_time();
    let mut value = INITIAL_TEST_PATTERN;

    loop {
        if timer::clock_time_exceed_us(tick, SPI_TRANSFER_PERIOD_US) {
            tick = timer::clock_time();
            let mut buf = [value];
            let _ = spi.transfer_in_place(&mut buf);
            let _ = spi.flush();

            let _ = board
                .led_y
                .set_state(PinState::from((buf[0] & LED_Y_MASK) != 0));
            let _ = board
                .led_w
                .set_state(PinState::from((buf[0] & LED_W_MASK) != 0));

            value = value.rotate_left(1);
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
