#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::{clock, interrupt, pac, pm, timer};

mod platform;

const SLEEP_MS: u32 = 2_000;
const RC_32K_HZ: u32 = 32_000;
const XTAL_32K_HZ: u32 = 32_768;

const LONG_PULSE_US: u32 = 240_000;
const SHORT_PULSE_US: u32 = 130_000;
const SERIES_GAP_US: u32 = 500_000;
const PRE_SLEEP_GAP_US: u32 = 1_000_000;

#[derive(Clone, Copy, Eq, PartialEq)]
enum SleepApi {
    SleepForMs,
    LongSleep32k,
}

#[derive(Clone, Copy)]
struct TestCase {
    clock: pm::Clock32kSource,
    mode: pm::SleepMode,
    api: SleepApi,
}

const TESTS: [TestCase; 8] = [
    TestCase {
        clock: pm::Clock32kSource::InternalRc,
        mode: pm::SleepMode::DeepSleepRetentionLow8K,
        api: SleepApi::LongSleep32k,
    },
    TestCase {
        clock: pm::Clock32kSource::ExternalCrystal,
        mode: pm::SleepMode::DeepSleepRetentionLow8K,
        api: SleepApi::LongSleep32k,
    },
    TestCase {
        clock: pm::Clock32kSource::InternalRc,
        mode: pm::SleepMode::DeepSleepRetentionLow16K,
        api: SleepApi::LongSleep32k,
    },
    TestCase {
        clock: pm::Clock32kSource::ExternalCrystal,
        mode: pm::SleepMode::DeepSleepRetentionLow16K,
        api: SleepApi::LongSleep32k,
    },
    TestCase {
        clock: pm::Clock32kSource::InternalRc,
        mode: pm::SleepMode::DeepSleepRetentionLow32K,
        api: SleepApi::LongSleep32k,
    },
    TestCase {
        clock: pm::Clock32kSource::ExternalCrystal,
        mode: pm::SleepMode::DeepSleepRetentionLow32K,
        api: SleepApi::LongSleep32k,
    },
    // DeepSleep at end: this may reset RAM-backed index.
    TestCase {
        clock: pm::Clock32kSource::InternalRc,
        mode: pm::SleepMode::DeepSleep,
        api: SleepApi::LongSleep32k,
    },
    TestCase {
        clock: pm::Clock32kSource::ExternalCrystal,
        mode: pm::SleepMode::DeepSleep,
        api: SleepApi::LongSleep32k,
    },
];

#[unsafe(no_mangle)]
static mut LAST_MODE_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut LAST_CLOCK_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut LAST_API_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut NEXT_TEST_INDEX: u8 = 0;
#[unsafe(no_mangle)]
static mut WAS_INITIALIZED: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    clock::init(clock::SysClock::Crystal16M);
    pm::sync_sys_tick_per_us();
    pm::init(pm::Clock32kSource::ExternalCrystal);
    let _ = interrupt::enable();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    drive_pin(&mut board.led_w, false);
    drive_pin(&mut board.led_y, false);

    if unsafe { WAS_INITIALIZED } == 0 {
        unsafe {
            WAS_INITIALIZED = 1;
            NEXT_TEST_INDEX = 0;
        }
    }

    loop {
        indicate_startup_state(&mut board);
        indicate_last_clock(&mut board);
        indicate_last_api(&mut board);
        delay_us(PRE_SLEEP_GAP_US);

        let case = TESTS[unsafe { NEXT_TEST_INDEX as usize % TESTS.len() }];
        let next = (unsafe { NEXT_TEST_INDEX as usize } + 1) % TESTS.len();
        unsafe {
            NEXT_TEST_INDEX = next as u8;
            LAST_MODE_RAW = case.mode.raw();
            LAST_CLOCK_RAW = match case.clock {
                pm::Clock32kSource::InternalRc => 1,
                pm::Clock32kSource::ExternalCrystal => 2,
            };
            LAST_API_RAW = match case.api {
                SleepApi::SleepForMs => 1,
                SleepApi::LongSleep32k => 2,
            };
        }

        match case.clock {
            pm::Clock32kSource::InternalRc => pm::pm_select_internal_32k_rc(),
            pm::Clock32kSource::ExternalCrystal => pm::pm_select_external_32k_crystal(),
        }

        match case.api {
            SleepApi::SleepForMs => {
                let _ = pm::sleep_for_ms(case.mode, pm::WakeupSource::TIMER, SLEEP_MS);
            }
            SleepApi::LongSleep32k => {
                let hz = match case.clock {
                    pm::Clock32kSource::InternalRc => RC_32K_HZ,
                    pm::Clock32kSource::ExternalCrystal => XTAL_32K_HZ,
                };
                let _ = pm::long_sleep_32k(
                    case.mode,
                    pm::WakeupSource::TIMER,
                    (SLEEP_MS.saturating_mul(hz)) / 1000,
                );
            }
        }
    }
}

fn indicate_startup_state(board: &mut Board) {
    let count = match pm::wake_origin() {
        pm::WakeOrigin::ColdBoot => {
            let last = unsafe { LAST_MODE_RAW };
            if last == pm::SleepMode::Suspend.raw() {
                6
            } else {
                1
            }
        }
        pm::WakeOrigin::DeepWake => 2,
        pm::WakeOrigin::DeepRetentionWake => {
            let last = unsafe { LAST_MODE_RAW };
            if last == pm::SleepMode::DeepSleepRetentionLow8K.raw() {
                3
            } else if last == pm::SleepMode::DeepSleepRetentionLow16K.raw() {
                4
            } else if last == pm::SleepMode::DeepSleepRetentionLow32K.raw() {
                5
            } else {
                3
            }
        }
    };
    blink_n(&mut board.led_w, count, LONG_PULSE_US);
}

fn indicate_last_clock(board: &mut Board) {
    delay_us(SERIES_GAP_US);
    let count = unsafe { LAST_CLOCK_RAW };
    let count = if count == 0 { 2 } else { count };
    blink_n(&mut board.led_y, count, LONG_PULSE_US);
}

fn indicate_last_api(board: &mut Board) {
    delay_us(SERIES_GAP_US);
    let count = unsafe { LAST_API_RAW };
    let count = if count == 0 { 2 } else { count };
    blink_n(&mut board.led_w, count, SHORT_PULSE_US);
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

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
