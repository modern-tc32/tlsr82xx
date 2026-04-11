#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tlsr82xx_hal::analog;
use tlsr82xx_hal::interrupt;
use tlsr82xx_hal::timer;

mod platform;

const IRQ_PERIOD_US: u32 = 200_000;
const IRQ_PERIOD_TICKS: u32 = IRQ_PERIOD_US * timer::SYS_TICK_PER_US;
const GPIO_PB_OUT: *mut u8 = 0x80058b as *mut u8;
const GPIO_PB_OEN: *mut u8 = 0x80058a as *mut u8;
const GPIO_PB_GPIO: *mut u8 = 0x80058e as *mut u8;
const PB4_MASK: u8 = 1 << 4;
const PB5_MASK: u8 = 1 << 5;

static mut MAIN_LAST_IRQ_COUNT: u32 = 0;
static mut VISIBLE_STATE: u8 = 0;
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    unsafe {
        core::ptr::write_volatile(&raw mut MAIN_LAST_IRQ_COUNT, 0);
        core::ptr::write_volatile(&raw mut VISIBLE_STATE, 0);
    }
    init_led_gpio_raw();
    set_white_raw();

    timer::configure_system_timer_periodic_irq(IRQ_PERIOD_TICKS);
    interrupt::enable();

    loop {
        let irq_count = timer::system_timer_irq_count();
        let last_irq_count = unsafe { core::ptr::read_volatile(&raw const MAIN_LAST_IRQ_COUNT) };
        if irq_count != last_irq_count {
            unsafe {
                core::ptr::write_volatile(&raw mut MAIN_LAST_IRQ_COUNT, irq_count);
            }
            let visible = (unsafe { core::ptr::read_volatile(&raw const VISIBLE_STATE) } & 1) ^ 1;
            unsafe {
                core::ptr::write_volatile(&raw mut VISIBLE_STATE, visible);
            }
            if visible != 0 {
                set_white_raw();
            } else {
                set_yellow_raw();
            }
        }
        core::hint::spin_loop();
    }
}

fn init_led_gpio_raw() {
    unsafe {
        let ds = analog::read(0xbf) | (PB4_MASK | PB5_MASK);
        analog::write(0xbf, ds);
        core::ptr::write_volatile(
            GPIO_PB_GPIO,
            core::ptr::read_volatile(GPIO_PB_GPIO.cast_const()) | (PB4_MASK | PB5_MASK),
        );
        core::ptr::write_volatile(
            GPIO_PB_OEN,
            core::ptr::read_volatile(GPIO_PB_OEN.cast_const()) & !(PB4_MASK | PB5_MASK),
        );
    }
}

fn set_white_raw() {
    unsafe {
        let out = core::ptr::read_volatile(GPIO_PB_OUT.cast_const());
        core::ptr::write_volatile(GPIO_PB_OUT, (out | PB5_MASK) & !PB4_MASK);
    }
}

fn set_yellow_raw() {
    unsafe {
        let out = core::ptr::read_volatile(GPIO_PB_OUT.cast_const());
        core::ptr::write_volatile(GPIO_PB_OUT, (out | PB4_MASK) & !PB5_MASK);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
