#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::{analog, clock, interrupt, pac, pm, startup, timer};

mod platform;

const SLEEP_MS: u32 = 2_000;
const RC_32K_HZ: u32 = 32_000;
const XTAL_32K_HZ: u32 = 32_768;

const LONG_PULSE_US: u32 = 240_000;
const SHORT_PULSE_US: u32 = 130_000;
const SERIES_GAP_US: u32 = 500_000;
const PRE_SLEEP_GAP_US: u32 = 1_000_000;
const FIRST_START_MARK_US: u32 = 3_000_000;
const ANA_PERSIST_STEP_REG: u8 = 0x3a;
const ANA_PERSIST_MAGIC_MASK: u8 = 0xF0;
const ANA_PERSIST_MAGIC_VALUE: u8 = 0xA0;

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
static mut LAST_TEST_INDEX_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut NEXT_TEST_INDEX: u8 = 0;
#[unsafe(no_mangle)]
static mut WAS_INITIALIZED: u8 = 0;
#[unsafe(no_mangle)]
static mut FORCE_COLD_BOOT_DISPLAY_ONCE: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_STAGE: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_LOOP_CNT: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_CASE_CUR: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_CASE_NEXT: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_CASE_COUNT: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_WAKE_ORIGIN: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_WAKE_FLAG: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_WAKE_SRC_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_IS_PAD_WAKE: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_MODE_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_CLOCK_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_NOW_TICK: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_TPU: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_WAKEUP_TICK: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_LAST_RET: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_PM_LONG_SUSPEND: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_TICK_CUR: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_TICK_32K_CUR: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_TICK_32K_CALIB: u16 = 0;
#[unsafe(no_mangle)]
static mut DBG_ERR: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_PERSIST_RAW: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    dbg_u32(&raw mut DBG_STAGE, 0x01);
    let _ = platform::init();
    dbg_u32(&raw mut DBG_STAGE, 0x02);
    clock::init(clock::SysClock::Crystal16M);
    pm::sync_sys_tick_per_us();
    pm::Pm::init(pm::Clock32kSource::ExternalCrystal);
    dbg_u32(&raw mut DBG_STAGE, 0x03);
    let _ = interrupt::enable();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    drive_pin(&mut board.led_w, false);
    drive_pin(&mut board.led_y, false);

    if unsafe { WAS_INITIALIZED } == 0 {
        let persisted = load_persisted_step();
        dbg_u8(&raw mut DBG_PERSIST_RAW, persisted);
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
        }
        if !has_magic {
            drive_pin(&mut board.led_w, true);
            drive_pin(&mut board.led_y, true);
            delay_us(FIRST_START_MARK_US);
            drive_pin(&mut board.led_w, false);
            drive_pin(&mut board.led_y, false);
            delay_us(SERIES_GAP_US);
        }
    }

    loop {
        dbg_inc(&raw mut DBG_LOOP_CNT);
        dbg_u32(&raw mut DBG_STAGE, 0x10);
        dbg_u8(
            &raw mut DBG_WAKE_ORIGIN,
            match pm::wake_origin() {
                pm::WakeOrigin::ColdBoot => 1,
                pm::WakeOrigin::DeepWake => 2,
                pm::WakeOrigin::DeepRetentionWake => 3,
            },
        );
        dbg_u8(&raw mut DBG_WAKE_FLAG, unsafe {
            startup::PM_STARTUP_DBG_WAKEUP_FLAG
        });
        dbg_u8(&raw mut DBG_WAKE_SRC_RAW, pm::wakeup_source_raw());
        dbg_u8(&raw mut DBG_IS_PAD_WAKE, pm::is_pad_wakeup() as u8);
        indicate_startup_state(&mut board);
        indicate_startup_wakeup_flag(&mut board);
        indicate_last_clock(&mut board);
        indicate_last_step(&mut board);
        delay_us(PRE_SLEEP_GAP_US);

        let idx = unsafe { NEXT_TEST_INDEX as usize % TESTS.len() };
        let case = TESTS[idx];
        let next = (unsafe { NEXT_TEST_INDEX as usize } + 1) % TESTS.len();
        unsafe {
            NEXT_TEST_INDEX = next as u8;
            LAST_TEST_INDEX_RAW = idx as u8;
            LAST_MODE_RAW = case.mode.raw();
            LAST_CLOCK_RAW = match case.clock {
                pm::Clock32kSource::InternalRc => 1,
                pm::Clock32kSource::ExternalCrystal => 2,
            };
        }
        persist_step(next as u8);
        dbg_u8(&raw mut DBG_CASE_CUR, idx as u8);
        dbg_u8(&raw mut DBG_CASE_NEXT, next as u8);
        dbg_u8(&raw mut DBG_MODE_RAW, case.mode.raw());
        dbg_u8(
            &raw mut DBG_CLOCK_RAW,
            match case.clock {
                pm::Clock32kSource::InternalRc => 1,
                pm::Clock32kSource::ExternalCrystal => 2,
            },
        );
        dbg_inc(&raw mut DBG_CASE_COUNT);

        dbg_u32(&raw mut DBG_STAGE, 0x20);
        pm::Pm::init(case.clock);
        dbg_u32(&raw mut DBG_STAGE, 0x21);
        let now = timer::clock_time();
        let tpu = timer::sys_tick_per_us();
        dbg_u32(&raw mut DBG_NOW_TICK, now);
        dbg_u32(&raw mut DBG_TPU, tpu);
        dbg_u32(
            &raw mut DBG_WAKEUP_TICK,
            now.wrapping_add(SLEEP_MS.saturating_mul(1000).saturating_mul(tpu)),
        );
        unsafe {
            DBG_PM_LONG_SUSPEND = startup::pm_long_suspend;
            DBG_TICK_CUR = startup::tick_cur;
            DBG_TICK_32K_CUR = startup::tick_32k_cur;
            DBG_TICK_32K_CALIB = startup::tick_32k_calib;
        }

        dbg_u32(&raw mut DBG_STAGE, 0x30);
        match case.api {
            SleepApi::SleepForMs => {
                let ret = pm::Pm::sleep_for_ms(case.mode, pm::WakeupSource::TIMER, SLEEP_MS);
                dbg_u32(&raw mut DBG_LAST_RET, ret.raw);
            }
            SleepApi::LongSleep32k => {
                let hz = match case.clock {
                    pm::Clock32kSource::InternalRc => RC_32K_HZ,
                    pm::Clock32kSource::ExternalCrystal => XTAL_32K_HZ,
                };
                let ret = pm::Pm::long_sleep_32k(
                    case.mode,
                    pm::WakeupSource::TIMER,
                    (SLEEP_MS.saturating_mul(hz)) / 1000,
                );
                dbg_u32(&raw mut DBG_LAST_RET, ret.raw);
            }
        }
        dbg_u32(&raw mut DBG_STAGE, 0x40);
    }
}

fn indicate_startup_state(board: &mut Board) {
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

fn indicate_startup_wakeup_flag(board: &mut Board) {
    delay_us(SERIES_GAP_US);
    let wakeup_flag = unsafe { startup::PM_STARTUP_DBG_WAKEUP_FLAG };
    let count = if wakeup_flag == 0 {
        1
    } else if wakeup_flag == 1 {
        2
    } else {
        3
    };
    blink_n(&mut board.led_w, count, SHORT_PULSE_US);
}

fn indicate_last_clock(board: &mut Board) {
    delay_us(SERIES_GAP_US);
    let count = unsafe { LAST_CLOCK_RAW };
    let count = if count == 0 { 2 } else { count };
    blink_n(&mut board.led_y, count, LONG_PULSE_US);
}

fn indicate_last_step(board: &mut Board) {
    delay_us(SERIES_GAP_US);
    let count = unsafe { LAST_TEST_INDEX_RAW.wrapping_add(1) };
    blink_n_custom(&mut board.led_y, count, SHORT_PULSE_US, SHORT_PULSE_US.saturating_mul(2));
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
fn dbg_u32(slot: *mut u32, value: u32) {
    unsafe {
        core::ptr::write_volatile(slot, value);
    }
}

#[inline(always)]
fn dbg_u8(slot: *mut u8, value: u8) {
    unsafe {
        core::ptr::write_volatile(slot, value);
    }
}

#[inline(always)]
fn dbg_inc(slot: *mut u32) {
    unsafe {
        let v = core::ptr::read_volatile(slot.cast_const());
        core::ptr::write_volatile(slot, v.wrapping_add(1));
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
