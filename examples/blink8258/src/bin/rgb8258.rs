#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tlsr82xx_hal::gpio::{GpioExt, PinFunction};
use tlsr82xx_hal::pac;
use tlsr82xx_hal::pwm::{Channel, PwmExt};
use tlsr82xx_hal::timer;

#[path = "../platform.rs"]
mod platform;

const PWM_PERIOD_TICKS: u16 = 48_000;
const BRIGHTNESS_MAX: u16 = 255;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        let _ = platform::drv_platform_init();
    }

    let peripherals = unsafe { pac::Peripherals::steal() };
    let mut pins = peripherals.gpio.split();
    pins.pc2.set_function(PinFunction::Pwm0);
    pins.pc3.set_function(PinFunction::Pwm1);
    pins.pc4.set_function(PinFunction::Pwm2);
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
