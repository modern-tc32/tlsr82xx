#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::{analog, clock, interrupt, pac, pm, startup, timer};

mod platform;

const SLEEP_MS: u32 = 3_000;
const RC_32K_HZ: u32 = 32_000;

const LONG_PULSE_US: u32 = 240_000;
const SHORT_PULSE_US: u32 = 130_000;
const SERIES_GAP_US: u32 = 500_000;
const PRE_SLEEP_GAP_US: u32 = 500_000;
const FIRST_START_MARK_US: u32 = 3_000_000;

const ANA_PERSIST_STEP_REG: u8 = 0x3a;
const ANA_PERSIST_MAGIC_MASK: u8 = 0xF0;
const ANA_PERSIST_MAGIC_VALUE: u8 = 0xA0;
const ANA_SLEEP_TICK0: u8 = 0x35;
const ANA_SLEEP_TICK_MAGIC: u8 = 0x39;
const ANA_SLEEP_TICK_MAGIC_VALUE: u8 = 0x5A;
const ANA_SLEEP_RETURN_MAGIC_VALUE: u8 = 0xA7;

#[derive(Clone, Copy)]
struct TestCase {
    clock: pm::Clock32kSource,
    mode: pm::SleepMode,
}

// TB03F on the current bench has no 32k XTAL fitted. Keep the pmled flow on
// the original long_sleep_32k API, but schedule only RC32K cases on hardware.
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
static mut DBG_LAST_RET: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_PERSIST_RAW: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_PERSIST_SET: u8 = 0;
#[unsafe(no_mangle)]
static mut DBG_SLEEP32K_PREV: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_SLEEP32K_NOW: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_SLEEP32K_DELTA: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_SLEEP_CALL_T0: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_SLEEP_CALL_T1: u32 = 0;
#[unsafe(no_mangle)]
static mut DBG_SLEEP_CALL_DELTA: u32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    dbg_u32(&raw mut DBG_STAGE, 0x01);
    let _ = platform::init();

    dbg_u32(&raw mut DBG_STAGE, 0x02);
    clock::init(clock::SysClock::Crystal16M);
    configure_pm_runtime(pm::Clock32kSource::InternalRc);
    update_sleep_delta_debug();
    let _ = interrupt::enable();

    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });
    drive_pin(&mut board.led_w, false);
    drive_pin(&mut board.led_y, false);
    disable_pad_wakeup_sources();

    if unsafe { WAS_INITIALIZED } == 0 {
        let persisted = load_persisted_step();
        let has_magic = (persisted & ANA_PERSIST_MAGIC_MASK) == ANA_PERSIST_MAGIC_VALUE;
        let persisted_idx = persisted & 0x0f;
        dbg_u8(&raw mut DBG_PERSIST_RAW, persisted);

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

        dbg_u32(&raw mut DBG_STAGE, 0x11);
        indicate_startup_state(&mut board);
        dbg_u32(&raw mut DBG_STAGE, 0x12);
        indicate_startup_wakeup_flag(&mut board);
        dbg_u32(&raw mut DBG_STAGE, 0x13);
        indicate_last_clock(&mut board);
        dbg_u32(&raw mut DBG_STAGE, 0x14);
        indicate_last_step(&mut board);
        dbg_u32(&raw mut DBG_STAGE, 0x15);
        delay_us(PRE_SLEEP_GAP_US);
        dbg_u32(&raw mut DBG_STAGE, 0x16);

        let idx = unsafe { NEXT_TEST_INDEX as usize % TESTS.len() };
        let case = TESTS[idx];
        let next = (idx + 1) % TESTS.len();
        unsafe {
            NEXT_TEST_INDEX = next as u8;
            LAST_TEST_INDEX_RAW = idx as u8;
            LAST_MODE_RAW = case.mode.raw();
            LAST_CLOCK_RAW = 1;
        }
        dbg_u8(&raw mut DBG_CASE_CUR, idx as u8);
        dbg_u8(&raw mut DBG_CASE_NEXT, next as u8);
        dbg_u8(&raw mut DBG_MODE_RAW, case.mode.raw());
        dbg_u8(&raw mut DBG_CLOCK_RAW, 1);
        dbg_inc(&raw mut DBG_CASE_COUNT);

        persist_step(next as u8);
        dbg_u8(&raw mut DBG_PERSIST_SET, load_persisted_step());

        dbg_u32(&raw mut DBG_STAGE, 0x20);
        configure_pm_runtime(case.clock);
        disable_pad_wakeup_sources();
        save_sleep_tick_marker();

        let t0_call = timer::clock_time();
        dbg_u32(&raw mut DBG_SLEEP_CALL_T0, t0_call);
        let ret = pm::Pm::long_sleep_32k(
            case.mode,
            pm::WakeupSource::TIMER,
            (SLEEP_MS * RC_32K_HZ) / 1000,
        );
        let t1_call = timer::clock_time();

        dbg_u32(&raw mut DBG_LAST_RET, ret.raw);
        dbg_u32(&raw mut DBG_SLEEP_CALL_T1, t1_call);
        dbg_u32(&raw mut DBG_SLEEP_CALL_DELTA, t1_call.wrapping_sub(t0_call));
        save_sleep_return_marker(ret.raw);
        dbg_u32(&raw mut DBG_STAGE, 0x40);
    }
}

fn configure_pm_runtime(source: pm::Clock32kSource) {
    pm::Pm::init(source);
    pm::set_wakeup_timing(pm::WakeupTiming {
        deep_r_delay_us: 1000,
        suspend_ret_r_delay_us: 1000,
    });
    pm::set_xtal_stable_timing(pm::XtalStableTiming {
        delay_us: 0x87,
        loop_count: 10,
        nop_count: 200,
    });
    pm::sync_sys_tick_per_us();
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
    analog::write(ANA_PERSIST_STEP_REG, ANA_PERSIST_MAGIC_VALUE | (next & 0x0f));
}

#[inline(always)]
fn save_sleep_tick_marker() {
    let t = pm::pm_get_32k_tick();
    analog::write(ANA_SLEEP_TICK0, (t & 0xff) as u8);
    analog::write(ANA_SLEEP_TICK0 + 1, ((t >> 8) & 0xff) as u8);
    analog::write(ANA_SLEEP_TICK0 + 2, ((t >> 16) & 0xff) as u8);
    analog::write(ANA_SLEEP_TICK0 + 3, ((t >> 24) & 0xff) as u8);
    analog::write(ANA_SLEEP_TICK_MAGIC, ANA_SLEEP_TICK_MAGIC_VALUE);
}

#[inline(always)]
fn load_sleep_tick_marker() -> Option<u32> {
    if analog::read(ANA_SLEEP_TICK_MAGIC) != ANA_SLEEP_TICK_MAGIC_VALUE {
        return None;
    }
    let b0 = analog::read(ANA_SLEEP_TICK0) as u32;
    let b1 = analog::read(ANA_SLEEP_TICK0 + 1) as u32;
    let b2 = analog::read(ANA_SLEEP_TICK0 + 2) as u32;
    let b3 = analog::read(ANA_SLEEP_TICK0 + 3) as u32;
    Some(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
}

#[inline(always)]
fn update_sleep_delta_debug() {
    let now = pm::pm_get_32k_tick();
    let prev = load_sleep_tick_marker().unwrap_or(0);
    dbg_u32(&raw mut DBG_SLEEP32K_PREV, prev);
    dbg_u32(&raw mut DBG_SLEEP32K_NOW, now);
    dbg_u32(
        &raw mut DBG_SLEEP32K_DELTA,
        if prev != 0 { now.wrapping_sub(prev) } else { 0 },
    );
}

#[inline(always)]
fn save_sleep_return_marker(value: u32) {
    analog::write(ANA_SLEEP_TICK0, (value & 0xff) as u8);
    analog::write(ANA_SLEEP_TICK0 + 1, ((value >> 8) & 0xff) as u8);
    analog::write(ANA_SLEEP_TICK0 + 2, ((value >> 16) & 0xff) as u8);
    analog::write(ANA_SLEEP_TICK0 + 3, ((value >> 24) & 0xff) as u8);
    analog::write(ANA_SLEEP_TICK_MAGIC, ANA_SLEEP_RETURN_MAGIC_VALUE);
}

#[inline(always)]
fn disable_pad_wakeup_sources() {
    analog::write(0x27, 0x00);
    analog::write(0x28, 0x00);
    analog::write(0x29, 0x00);
    analog::write(0x2a, 0x00);
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
