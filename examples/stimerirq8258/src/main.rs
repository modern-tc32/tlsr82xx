#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::interrupt::{self, Irq};
use tlsr82xx_hal::timer;

mod platform;

const IRQ_PERIOD_US: u32 = 1_000_000;
const TIMER0_PERIOD_TICKS: u32 = IRQ_PERIOD_US * timer::SYS_TICK_PER_US;
const REG_GPIO_PB_DATA: *mut u8 = 0x0080_058b as *mut u8;
const MAIN_POLL_US: u32 = 80_000;

static mut MAIN_LAST_PHASE: u8 = 0;

unsafe extern "C" {
    static stimerirq8258_irq_phase: u8;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code")]
pub extern "C" fn stimerirq8258_rust_handler(phase: *mut u8) {
    unsafe {
        let next = core::ptr::read_volatile(phase.cast_const()) ^ 1;
        core::ptr::write_volatile(phase, next);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    let _board = Board::from_peripherals(unsafe { tlsr82xx_hal::pac::Peripherals::steal() });

    timer::clear_timer0_status();
    interrupt::clear_irq(Irq::Timer0);
    timer::set_timer0_mode_sysclk();
    timer::set_timer0_tick(0);
    timer::set_timer0_irq_capture(TIMER0_PERIOD_TICKS);
    timer::enable_timer0();
    interrupt::enable_irq(Irq::Timer0);
    interrupt::enable();

    set_pb_leds(false, true);
    let mut next_poll = timer::clock_time();
    loop {
        if !timer::clock_time_exceed_us(next_poll, MAIN_POLL_US) {
            core::hint::spin_loop();
            continue;
        }
        next_poll = timer::clock_time();
        let phase = read_irq_phase();
        if phase as u8 != read_main_last_phase() {
            write_main_last_phase(phase as u8);
            set_pb_leds(phase, !phase);
        }
        core::hint::spin_loop();
    }
}

fn read_irq_phase() -> bool {
    unsafe { core::ptr::read_volatile(&raw const stimerirq8258_irq_phase) != 0 }
}

fn read_main_last_phase() -> u8 {
    unsafe { core::ptr::read_volatile(&raw const MAIN_LAST_PHASE) }
}

fn write_main_last_phase(value: u8) {
    unsafe {
        core::ptr::write_volatile(&raw mut MAIN_LAST_PHASE, value);
    }
}

fn set_pb_leds(yellow: bool, white: bool) {
    let mut pb = unsafe { core::ptr::read_volatile(REG_GPIO_PB_DATA.cast_const()) };
    if yellow {
        pb |= 1 << 4;
    } else {
        pb &= !(1 << 4);
    }
    if white {
        pb |= 1 << 5;
    } else {
        pb &= !(1 << 5);
    }
    unsafe {
        core::ptr::write_volatile(REG_GPIO_PB_DATA, pb);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
