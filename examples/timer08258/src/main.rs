#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tlsr82xx_hal::interrupt::{self, Irq};
use tlsr82xx_hal::timer;

mod platform;

const IRQ_PERIOD_US: u32 = 200_000;
const GPIO_PB_OUT: *mut u8 = 0x80058b as *mut u8;
const GPIO_PB_OEN: *mut u8 = 0x80058a as *mut u8;
const GPIO_PB_GPIO: *mut u8 = 0x80058e as *mut u8;
const ANA_GPIO_DS: u8 = 0xbf;
const PB4_MASK: u8 = 1 << 4;
const PB5_MASK: u8 = 1 << 5;

static mut LED_STATE: u8 = 0;

tlsr82xx_hal::define_ram_void_handler! {
    unsafe extern "C" fn timer0_led_irq() {
        let next = (core::ptr::read_volatile(&raw const LED_STATE) & 1) ^ 1;
        core::ptr::write_volatile(&raw mut LED_STATE, next);

        let out = core::ptr::read_volatile(GPIO_PB_OUT.cast_const());
        let next_out = if next != 0 {
            (out | PB5_MASK) & !PB4_MASK
        } else {
            (out | PB4_MASK) & !PB5_MASK
        };
        core::ptr::write_volatile(GPIO_PB_OUT, next_out);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::init();
    unsafe {
        core::ptr::write_volatile(&raw mut LED_STATE, 1);
    }
    init_led_gpio_raw();
    set_white_raw();

    interrupt::clear_irq(Irq::Timer0);
    timer::configure_timer0_periodic_irq(timer::timer0_sysclk_ticks_from_us(IRQ_PERIOD_US));
    timer::register_timer0_irq_callback(tlsr82xx_hal::ram_void_handler!(timer0_led_irq));
    interrupt::enable_irq(Irq::Timer0);
    interrupt::enable();

    loop {
        core::hint::spin_loop();
    }
}

fn init_led_gpio_raw() {
    unsafe {
        let ds = tlsr82xx_hal::analog::read(ANA_GPIO_DS) | (PB4_MASK | PB5_MASK);
        tlsr82xx_hal::analog::write(ANA_GPIO_DS, ds);
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

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
