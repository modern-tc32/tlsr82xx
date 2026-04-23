#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::{analog, clock, interrupt, pac, pm, timer};

mod platform;

const SLEEP_MS: u32 = 3_000;

const LONG_PULSE_US: u32 = 240_000;
const SHORT_PULSE_US: u32 = 130_000;
const SERIES_GAP_US: u32 = 500_000;
const PRE_SLEEP_GAP_US: u32 = 500_000;
const FIRST_START_MARK_US: u32 = 3_000_000;

const ANA_PERSIST_STEP_REG: u8 = 0x3a;
const ANA_PERSIST_MAGIC_MASK: u8 = 0xF0;
const ANA_PERSIST_MAGIC_VALUE: u8 = 0xA0;

#[derive(Clone, Copy)]
struct TestCase {
    clock: pm::Clock32kSource,
    mode: pm::SleepMode,
}

// TB03F on the current bench has no 32k XTAL fitted, so schedule only RC32K
// cases on hardware.
const TESTS: [TestCase; 4] = [
    TestCase {
        clock: pm::Clock32kSource::InternalRc,
        mode: pm::SleepMode::DeepSleepRetentionLow8K,
    },
    TestCase {
        clock: pm::Clock32kSource::InternalRc,
        mode: pm::SleepMode::DeepSleepRetentionLow16K,
    },
    TestCase {
        clock: pm::Clock32kSource::InternalRc,
        mode: pm::SleepMode::DeepSleepRetentionLow32K,
    },
    TestCase {
        clock: pm::Clock32kSource::InternalRc,
        mode: pm::SleepMode::DeepSleep,
    },
];

#[unsafe(no_mangle)]
static mut LAST_MODE_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut LAST_CLOCK_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut LAST_TEST_INDEX_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut NEXT_TEST_INDEX: u8 = 0;
#[unsafe(no_mangle)]
static mut WAS_INITIALIZED: u8 = 0;
#[unsafe(no_mangle)]
static mut FORCE_COLD_BOOT_DISPLAY_ONCE: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();

    clock::init(clock::SysClock::Crystal16M);
    let mut power = pm::Pm::init(pm::Config::internal_rc());
    let _ = interrupt::enable();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    drive_pin(&mut board.led_w, false);
    drive_pin(&mut board.led_y, false);

    if unsafe { WAS_INITIALIZED } == 0 {
        let persisted = load_persisted_step();
        let has_magic = (persisted & ANA_PERSIST_MAGIC_MASK) == ANA_PERSIST_MAGIC_VALUE;
        let persisted_idx = persisted & 0x0f;

        unsafe {
            WAS_INITIALIZED = 1;
            NEXT_TEST_INDEX = if has_magic {
                persisted_idx % (TESTS.len() as u8)
            } else {
                0
            };
            FORCE_COLD_BOOT_DISPLAY_ONCE = if has_magic { 0 } else { 1 };
        }

        if !has_magic {
            persist_step(0);
            drive_pin(&mut board.led_w, true);
            drive_pin(&mut board.led_y, true);
            delay_us(FIRST_START_MARK_US);
            drive_pin(&mut board.led_w, false);
            drive_pin(&mut board.led_y, false);
            delay_us(SERIES_GAP_US);
        }
    }

    loop {
        board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
        drive_pin(&mut board.led_w, false);
        drive_pin(&mut board.led_y, false);

        indicate_startup_state(&mut board, power.wake_info());
        indicate_last_clock(&mut board);
        indicate_last_step(&mut board);
        delay_us(PRE_SLEEP_GAP_US);

        let idx = unsafe { NEXT_TEST_INDEX as usize % TESTS.len() };
        let case = TESTS[idx];
        let next = (idx + 1) % TESTS.len();
        unsafe {
            NEXT_TEST_INDEX = next as u8;
            LAST_TEST_INDEX_RAW = idx as u8;
            LAST_MODE_RAW = case.mode.raw();
            LAST_CLOCK_RAW = 1;
        }

        persist_step(next as u8);

        power.reconfigure(config_for(case.clock));

        let _ = power.sleep_ms(case.mode, pm::WakeupSource::TIMER, SLEEP_MS);
    }
}

fn config_for(source: pm::Clock32kSource) -> pm::Config {
    match source {
        pm::Clock32kSource::InternalRc => pm::Config::internal_rc(),
        pm::Clock32kSource::ExternalCrystal => pm::Config::external_crystal(),
    }
}

fn indicate_startup_state(board: &mut Board, wake: pm::WakeInfo) {
    let force_cold_boot = unsafe {
        if FORCE_COLD_BOOT_DISPLAY_ONCE != 0 {
            FORCE_COLD_BOOT_DISPLAY_ONCE = 0;
            true
        } else {
            false
        }
    };

    if force_cold_boot {
        blink_n(&mut board.led_w, 1, LONG_PULSE_US);
        return;
    }

    let count = match wake.origin {
        pm::WakeOrigin::ColdBoot => 1,
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
    let count = match unsafe { LAST_CLOCK_RAW } {
        2 => 2,
        _ => 1,
    };
    blink_n(&mut board.led_y, count, LONG_PULSE_US);
}

fn indicate_last_step(board: &mut Board) {
    delay_us(SERIES_GAP_US);
    let count = unsafe { NEXT_TEST_INDEX.wrapping_add(1) };
    blink_n_custom(&mut board.led_y, count, SHORT_PULSE_US, SHORT_PULSE_US * 2);
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

#[inline(always)]
fn load_persisted_step() -> u8 {
    analog::read(ANA_PERSIST_STEP_REG)
}

#[inline(always)]
fn persist_step(next: u8) {
    analog::write(
        ANA_PERSIST_STEP_REG,
        ANA_PERSIST_MAGIC_VALUE | (next & 0x0f),
    );
}

fn blink_n<P: OutputPin>(pin: &mut P, count: u8, pulse_us: u32) {
    blink_n_custom(pin, count, pulse_us, pulse_us);
}

fn blink_n_custom<P: OutputPin>(pin: &mut P, count: u8, on_us: u32, off_us: u32) {
    let mut i = 0u8;
    while i < count {
        drive_pin(pin, true);
        delay_us(on_us);
        drive_pin(pin, false);
        delay_us(off_us);
        i = i.wrapping_add(1);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
