#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::OutputPin;
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::pm::{self, SleepMode, WakeupSource};
use tlsr82xx_hal::startup::StartupState;
use tlsr82xx_hal::timer;

mod platform;

const BOOT_WHITE_US: u32 = 3_000_000;
const YELLOW_ON_US: u32 = 1_000_000;
const SLEEP_MS: u32 = 1_000;
const SLEEP_MODE: SleepMode = SleepMode::DeepSleepRetentionLow32K;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let state = platform::init();
    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });

    match state {
        StartupState::Boot => {
            set_white(&mut board);
            busy_wait_us(BOOT_WHITE_US);
            set_all_off(&mut board);
        }
        StartupState::Deep | StartupState::DeepRetention => {
            set_yellow(&mut board);
            busy_wait_us(YELLOW_ON_US);
            set_all_off(&mut board);
        }
    }

    loop {
        let _ = pm::long_sleep_32k(SLEEP_MODE, WakeupSource::TIMER, SLEEP_MS * 32);
    }
}

fn busy_wait_us(delay_us: u32) {
    let start = timer::clock_time();
    while !timer::clock_time_exceed_us(start, delay_us) {
        core::hint::spin_loop();
    }
}

fn set_white(board: &mut Board) {
    let _ = board.led_y.set_low();
    let _ = board.led_w.set_high();
}

fn set_yellow(board: &mut Board) {
    let _ = board.led_w.set_low();
    let _ = board.led_y.set_high();
}

fn set_all_off(board: &mut Board) {
    let _ = board.led_y.set_low();
    let _ = board.led_w.set_low();
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
