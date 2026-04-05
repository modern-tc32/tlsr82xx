#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tlsr82xx_hal::pac;
use tlsr82xx_hal::pwm::{Channel, PwmExt};
use tlsr82xx_hal::timer;

#[path = "../platform.rs"]
mod platform;

const PORT_C: usize = 2;
const GPIO_BASE: usize = 0x0080_0580;
const MUX_BASE: usize = 0x0080_05a8;
const PWM_PERIOD_TICKS: u16 = 48_000;
const BRIGHTNESS_MAX: u16 = 255;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        let _ = platform::drv_platform_init();
    }

    configure_tb03f_rgb_pin(2);
    configure_tb03f_rgb_pin(3);
    configure_tb03f_rgb_pin(4);

    let peripherals = unsafe { pac::Peripherals::steal() };
    let mut pwm = peripherals.pwm.constrain();
    pwm.configure(Channel::Pwm0, PWM_PERIOD_TICKS, 0);
    pwm.configure(Channel::Pwm1, PWM_PERIOD_TICKS, 0);
    pwm.configure(Channel::Pwm2, PWM_PERIOD_TICKS, 0);
    pwm.enable(Channel::Pwm0);
    pwm.enable(Channel::Pwm1);
    pwm.enable(Channel::Pwm2);

    let mut phase = 0u16;
    let mut tick = timer::clock_time();

    loop {
        if timer::clock_time_exceed_us(tick, 10_000) {
            tick = timer::clock_time();
            phase = phase.wrapping_add(1) % (BRIGHTNESS_MAX * 6);
            let rgb = wheel(phase);
            pwm.set_duty_8bit(Channel::Pwm0, rgb.r);
            pwm.set_duty_8bit(Channel::Pwm1, rgb.g);
            pwm.set_duty_8bit(Channel::Pwm2, rgb.b);
        }
    }
}

fn configure_tb03f_rgb_pin(bit: usize) {
    let gpio_func = (GPIO_BASE + PORT_C * 8 + 0x06) as *mut u8;
    let mux = (MUX_BASE + PORT_C * 2 + (bit / 4)) as *mut u8;
    let shift = (3 - (bit % 4)) * 2;
    let field_mask = !(0b11u8 << shift);
    let bit_mask = 1u8 << bit;

    unsafe {
        let mux_value = core::ptr::read_volatile(mux.cast_const()) & field_mask;
        core::ptr::write_volatile(mux, mux_value);

        let gpio_func_value = core::ptr::read_volatile(gpio_func.cast_const()) & !bit_mask;
        core::ptr::write_volatile(gpio_func, gpio_func_value);
    }
}

struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

fn wheel(phase: u16) -> Rgb {
    let segment = (phase / BRIGHTNESS_MAX) % 6;
    let offset = (phase % BRIGHTNESS_MAX) as u8;
    let max = BRIGHTNESS_MAX as u8;

    match segment {
        0 => Rgb {
            r: max,
            g: offset,
            b: 0,
        },
        1 => Rgb {
            r: max.saturating_sub(offset),
            g: max,
            b: 0,
        },
        2 => Rgb {
            r: 0,
            g: max,
            b: offset,
        },
        3 => Rgb {
            r: 0,
            g: max.saturating_sub(offset),
            b: max,
        },
        4 => Rgb {
            r: offset,
            g: 0,
            b: max,
        },
        _ => Rgb {
            r: max,
            g: 0,
            b: max.saturating_sub(offset),
        },
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
