#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::pm::Clock32kSource;
use tlsr82xx_hal::pm::{self, SleepMode, WakeupSource};
use tlsr82xx_hal::startup::StartupState;
use tlsr82xx_hal::timer;

mod platform;

const BOOT_WHITE_US: u32 = 2_000_000;
const SLEEP_MS: u32 = 1_200;
const WAKE_BLINK_US: u32 = 220_000;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    pm::init(Clock32kSource::InternalRc);
    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });

    match pm::state() {
        StartupState::Boot => {
            drive_pin(&mut board.led_y, false);
            drive_pin(&mut board.led_w, true);
            busy_wait_us(BOOT_WHITE_US);
            drive_pin(&mut board.led_w, false);
        }
        StartupState::Deep => {
            drive_pin(&mut board.led_w, false);
            drive_pin(&mut board.led_y, true);
            busy_wait_us(WAKE_BLINK_US);
            drive_pin(&mut board.led_y, false);
        }
        StartupState::DeepRetention => {
            drive_pin(&mut board.led_w, false);
            drive_pin(&mut board.led_y, true);
            busy_wait_us(WAKE_BLINK_US);
            drive_pin(&mut board.led_y, false);
        }
    }

    loop {
        let _ = pm::sleep_for_ms(SleepMode::DeepSleep, WakeupSource::TIMER, SLEEP_MS);
    }
}

fn drive_pin<P: OutputPin>(pin: &mut P, high: bool) {
    let _ = pin.set_state(PinState::from(high));
}

fn busy_wait_us(delay_us: u32) {
    let start = timer::clock_time();
    while !timer::clock_time_exceed_us(start, delay_us) {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
