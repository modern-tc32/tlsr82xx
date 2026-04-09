#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::OutputPin;
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::gpio::GpioExt;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;
use tlsr82xx_hal::uart::{self, Config, UartExt};

mod platform;

const BAUDRATE: u32 = 115_200;
const TEST_PERIOD_US: u32 = 1_000_000;
const LOOPBACK_TIMEOUT_US: u32 = 50_000;
const TEST_BYTE: u8 = b'U';

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    let peripherals = unsafe { pac::Peripherals::steal() };
    let mut pins = peripherals.gpio.split();
    uart::apply_pins(&mut pins.pb1, &mut pins.pa0);
    let mut board = Board::from_pins(pins);
    let mut uart = peripherals.uart.constrain();
    uart.configure(Config::new(BAUDRATE));

    let _ = board.led_y.set_low();
    let _ = board.led_w.set_low();
    let mut white_on = false;
    let mut yellow_on = true;
    let mut no_echo_div = 0u8;

    let mut next_tick = timer::clock_time();
    loop {
        if !timer::clock_time_exceed_us(next_tick, TEST_PERIOD_US) {
            core::hint::spin_loop();
            continue;
        }
        next_tick = timer::clock_time();

        if !uart.try_write_byte(TEST_BYTE, 20_000) {
            // TX path blocked: fast yellow (toggle every test period).
            yellow_on = !yellow_on;
            let _ = if yellow_on {
                board.led_y.set_high()
            } else {
                board.led_y.set_low()
            };
            white_on = false;
            let _ = board.led_w.set_low();
            continue;
        }

        if wait_for_echo(&mut uart, TEST_BYTE, LOOPBACK_TIMEOUT_US) {
            // Echo path works: white toggles every second.
            white_on = !white_on;
            let _ = if white_on {
                board.led_w.set_high()
            } else {
                board.led_w.set_low()
            };
            yellow_on = false;
            let _ = board.led_y.set_low();
            no_echo_div = 0;
        } else {
            // TX accepted but no loopback echo: slow yellow (toggle every 2 seconds).
            no_echo_div ^= 1;
            if no_echo_div != 0 {
                yellow_on = !yellow_on;
                let _ = if yellow_on {
                    board.led_y.set_high()
                } else {
                    board.led_y.set_low()
                };
            }
            white_on = false;
            let _ = board.led_w.set_low();
        }
    }
}

fn wait_for_echo(uart: &mut tlsr82xx_hal::uart::Uart, expected: u8, timeout_us: u32) -> bool {
    let deadline = timer::clock_time();
    while !timer::clock_time_exceed_us(deadline, timeout_us) {
        if uart.read_ready() {
            return uart.read_byte() == expected;
        }
        core::hint::spin_loop();
    }
    false
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
