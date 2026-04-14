#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::{clock, interrupt, pac, pm, timer};

mod platform;

const BOOT_WHITE_US: u32 = 300_000;
const SLEEP_MS: u32 = 600;
const SLEEP_TICKS_32K_PER_MS: u32 = 32;
const WAKE_BLINK_US: u32 = 120_000;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    clock::init(clock::SysClock::Crystal16M);
    pm::sync_sys_tick_per_us();
    pm::init(pm::Clock32kSource::InternalRc);
    let _ = interrupt::enable();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    drive_pin(&mut board.led_y, false);
    drive_pin(&mut board.led_w, false);

    // White LED only on true cold boot.
    if pm::is_cold_boot() {
        drive_pin(&mut board.led_w, true);
        delay_us(BOOT_WHITE_US);
        drive_pin(&mut board.led_w, false);
    }

    loop {
        // Mark each wake with a short yellow pulse.
        drive_pin(&mut board.led_y, true);
        delay_us(WAKE_BLINK_US);
        drive_pin(&mut board.led_y, false);

        // Enter deep sleep and wake by timer.
        let _ = pm::long_sleep_32k(
            pm::SleepMode::DeepSleep,
            pm::WakeupSource::TIMER,
            SLEEP_MS.saturating_mul(SLEEP_TICKS_32K_PER_MS),
        );
    }
}

fn drive_pin<P: OutputPin>(pin: &mut P, high: bool) {
    let _ = pin.set_state(PinState::from(high));
}

fn delay_us(duration_us: u32) {
    let started = timer::clock_time();
    while !timer::clock_time_exceed_us(started, duration_us) {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
