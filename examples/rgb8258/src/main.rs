#![no_std]
#![no_main]

use core::panic::PanicInfo;

use embedded_hal::pwm::SetDutyCycle;
use tlsr82xx_boards::tb03f;
use tlsr82xx_hal::gpio::GpioExt;
use tlsr82xx_hal::pac;
use tlsr82xx_hal::pwm::{PwmExt, PWM_DUTY_MAX_8BIT};
use tlsr82xx_hal::timer;

mod platform;

const PWM_PERIOD_TICKS: u16 = 48_000;
const RGB_UPDATE_PERIOD_US: u32 = 10_000;
const COLOR_WHEEL_SEGMENTS: u16 = 6;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let _ = platform::drv_platform_init();

    let peripherals = unsafe { pac::Peripherals::steal() };
    let mut pins = peripherals.gpio.split();
    tb03f::configure_rgb_pins(&mut pins);
    let pwm = peripherals.pwm.constrain();
    let mut channels = pwm.split();
    channels.pwm0.configure(PWM_PERIOD_TICKS, 0);
    channels.pwm1.configure(PWM_PERIOD_TICKS, 0);
    channels.pwm2.configure(PWM_PERIOD_TICKS, 0);
    channels.pwm0.enable();
    channels.pwm1.enable();
    channels.pwm2.enable();

    let mut phase = 0u16;
    let mut tick = timer::clock_time();

    loop {
        if timer::clock_time_exceed_us(tick, RGB_UPDATE_PERIOD_US) {
            tick = timer::clock_time();
            phase = phase.wrapping_add(1) % (PWM_DUTY_MAX_8BIT * COLOR_WHEEL_SEGMENTS);
            let rgb = wheel(phase);
            let _ = channels
                .pwm0
                .set_duty_cycle_fraction(u16::from(rgb.r), PWM_DUTY_MAX_8BIT);
            let _ = channels
                .pwm1
                .set_duty_cycle_fraction(u16::from(rgb.g), PWM_DUTY_MAX_8BIT);
            let _ = channels
                .pwm2
                .set_duty_cycle_fraction(u16::from(rgb.b), PWM_DUTY_MAX_8BIT);
        }
    }
}

struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

fn wheel(phase: u16) -> Rgb {
    let segment = (phase / PWM_DUTY_MAX_8BIT) % COLOR_WHEEL_SEGMENTS;
    let offset = (phase % PWM_DUTY_MAX_8BIT) as u8;
    let max = PWM_DUTY_MAX_8BIT as u8;

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
