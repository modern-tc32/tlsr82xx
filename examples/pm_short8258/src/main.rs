#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::{analog, clock, interrupt, pac, pm, timer};

mod platform;

const SLEEP_MS: u32 = 2_000;
const LONG_PULSE_US: u32 = 240_000;
const SERIES_GAP_US: u32 = 500_000;

const TESTS: [pm::SleepMode; 4] = [
    pm::SleepMode::DeepSleepRetentionLow8K,
    pm::SleepMode::DeepSleepRetentionLow16K,
    pm::SleepMode::DeepSleepRetentionLow32K,
    pm::SleepMode::DeepSleep,
];

#[unsafe(no_mangle)]
static mut WAS_INITIALIZED: u8 = 0;
#[unsafe(no_mangle)]
static mut NEXT_STEP: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    clock::init(clock::SysClock::Crystal16M);
    let mut power = pm::Pm::init(pm::Config::internal_rc());
    let _ = interrupt::enable();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    drive_pin(&mut board.led_w, false);
    drive_pin(&mut board.led_y, false);
    disable_pad_wakeup_sources();

    unsafe {
        if WAS_INITIALIZED == 0 {
            WAS_INITIALIZED = 1;
            NEXT_STEP = 0;
        }
    }

    let step = unsafe { NEXT_STEP as usize % TESTS.len() };
    let mode = TESTS[step];

    blink_n(
        &mut board.led_w,
        (step as u8).wrapping_add(1),
        LONG_PULSE_US,
    );
    delay_us(SERIES_GAP_US);

    // Retention wakes keep SRAM, so advance the step before sleep.
    unsafe {
        NEXT_STEP = ((step + 1) % TESTS.len()) as u8;
    }

    let _ = power.sleep_ms_short(mode, pm::WakeupSource::TIMER, SLEEP_MS);

    loop {
        core::hint::spin_loop();
    }
}

fn drive_pin<P: OutputPin>(pin: &mut P, high: bool) {
    let _ = pin.set_state(PinState::from(high));
}

fn blink_n<P: OutputPin>(pin: &mut P, count: u8, pulse_us: u32) {
    let mut i = 0u8;
    while i < count {
        drive_pin(pin, true);
        delay_us(pulse_us);
        drive_pin(pin, false);
        delay_us(pulse_us);
        i = i.wrapping_add(1);
    }
}

fn delay_us(duration_us: u32) {
    let started = timer::clock_time();
    while !timer::clock_time_exceed_us(started, duration_us) {
        core::hint::spin_loop();
    }
}

#[inline(always)]
fn disable_pad_wakeup_sources() {
    analog::write(0x27, 0x00);
    analog::write(0x28, 0x00);
    analog::write(0x29, 0x00);
    analog::write(0x2a, 0x00);
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
