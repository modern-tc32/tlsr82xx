#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::digital::{OutputPin, PinState};
use tlsr82xx_boards::tb03f::Board;
use tlsr82xx_hal::interrupt::{self, Irq};
use tlsr82xx_hal::pac;
use tlsr82xx_hal::timer;

mod platform;

const IRQ_PERIOD_US: u32 = 1_000_000;
const TIMER0_PERIOD_TICKS: u32 = IRQ_PERIOD_US * timer::SYS_TICK_PER_US;
const MAIN_POLL_US: u32 = 80_000;

static mut MAIN_LAST_PHASE: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    let mut board = Board::from_peripherals(unsafe { pac::Peripherals::steal() });

    interrupt::clear_irq(Irq::Timer0);
    timer::configure_timer0_periodic_irq(TIMER0_PERIOD_TICKS);
    interrupt::enable_irq(Irq::Timer0);
    interrupt::enable();

    let _ = board.led_y.set_state(PinState::from(false));
    let _ = board.led_w.set_state(PinState::from(true));
    let mut next_poll = timer::clock_time();
    loop {
        if !timer::clock_time_exceed_us(next_poll, MAIN_POLL_US) {
            core::hint::spin_loop();
            continue;
        }
        next_poll = timer::clock_time();
        let phase = timer::timer0_irq_phase();
        if phase as u8 != read_main_last_phase() {
            write_main_last_phase(phase as u8);
            let _ = board.led_y.set_state(PinState::from(phase));
            let _ = board.led_w.set_state(PinState::from(!phase));
        }
        core::hint::spin_loop();
    }
}

fn read_main_last_phase() -> u8 {
    unsafe { core::ptr::read_volatile(&raw const MAIN_LAST_PHASE) }
}

fn write_main_last_phase(value: u8) {
    unsafe {
        core::ptr::write_volatile(&raw mut MAIN_LAST_PHASE, value);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
