use core::convert::Infallible;

use embedded_hal::pwm::{ErrorType, SetDutyCycle};

use crate::pac;

const RESET_BASE: usize = 0x0080_0060;
const PWM_ENABLE_REG: usize = 0x0080_0780;
const PWM0_ENABLE_REG: usize = 0x0080_0781;
const PWM_CLK_REG: usize = 0x0080_0782;
const PWM0_MODE_REG: usize = 0x0080_0783;
const PWM_INVERT_REG: usize = 0x0080_0784;
const PWM_POL_REG: usize = 0x0080_0786;
const PWM_CYCLE_BASE: usize = 0x0080_0794;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Channel {
    Pwm0 = 0,
    Pwm1 = 1,
    Pwm2 = 2,
    Pwm3 = 3,
    Pwm4 = 4,
    Pwm5 = 5,
}

impl Channel {
    #[inline(always)]
    const fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    const fn bit(self) -> u8 {
        1u8 << (self as u8)
    }
}

pub trait PwmExt {
    fn constrain(self) -> Pwm;
}

impl PwmExt for pac::Pwm {
    fn constrain(self) -> Pwm {
        Pwm::new(self)
    }
}

pub struct Pwm {
    _inner: pac::Pwm,
    period_ticks: [u16; 6],
}

pub struct Channels {
    pub pwm0: PwmChannel,
    pub pwm1: PwmChannel,
    pub pwm2: PwmChannel,
    pub pwm3: PwmChannel,
    pub pwm4: PwmChannel,
    pub pwm5: PwmChannel,
}

pub struct PwmChannel {
    channel: Channel,
    period_ticks: u16,
}

impl Pwm {
    pub fn new(inner: pac::Pwm) -> Self {
        let pwm = Self {
            _inner: inner,
            period_ticks: [0; 6],
        };
        pwm.enable_peripheral();
        pwm
    }

    #[inline(always)]
    fn cycle_reg(channel: Channel) -> *mut u32 {
        (PWM_CYCLE_BASE + channel.index() * 4) as *mut u32
    }

    #[inline(always)]
    fn enable_peripheral(&self) {
        unsafe {
            let clk_en0 = (RESET_BASE + 0x03) as *mut u8;
            let rst0 = RESET_BASE as *mut u8;

            core::ptr::write_volatile(
                clk_en0,
                core::ptr::read_volatile(clk_en0.cast_const()) | (1 << 4),
            );
            core::ptr::write_volatile(rst0, 1 << 4);
            core::ptr::write_volatile(rst0, 0);
        }
    }

    pub fn set_clock(&mut self, system_clock_hz: u32, pwm_clock_hz: u32) {
        let divider = system_clock_hz / pwm_clock_hz;
        let value = divider
            .checked_sub(1)
            .expect("pwm_clock_hz must be <= system_clock_hz");
        let value = u8::try_from(value).expect("PWM clock divider does not fit in 8 bits");
        unsafe {
            core::ptr::write_volatile(PWM_CLK_REG as *mut u8, value);
        }
    }

    pub fn configure(&mut self, channel: Channel, cycle_ticks: u16, duty_ticks: u16) {
        let duty_ticks = duty_ticks.min(cycle_ticks);
        self.period_ticks[channel.index()] = cycle_ticks;
        if channel == Channel::Pwm0 {
            unsafe {
                core::ptr::write_volatile(PWM0_MODE_REG as *mut u8, 0);
            }
        }
        unsafe {
            core::ptr::write_volatile(
                Self::cycle_reg(channel),
                u32::from(duty_ticks) | (u32::from(cycle_ticks) << 16),
            );
        }
    }

    pub fn enable(&mut self, channel: Channel) {
        match channel {
            Channel::Pwm0 => self
                ._inner
                .pwm0_enable()
                .modify(|r, w| unsafe { w.bits(r.bits() | 0x01) }),
            _ => self
                ._inner
                .pwm_enable()
                .modify(|r, w| unsafe { w.bits(r.bits() | channel.bit()) }),
        };
    }

    pub fn disable(&mut self, channel: Channel) {
        match channel {
            Channel::Pwm0 => self
                ._inner
                .pwm0_enable()
                .modify(|r, w| unsafe { w.bits(r.bits() & !0x01) }),
            _ => self
                ._inner
                .pwm_enable()
                .modify(|r, w| unsafe { w.bits(r.bits() & !channel.bit()) }),
        };
    }

    pub fn set_duty_ticks(&mut self, channel: Channel, duty_ticks: u16) {
        let period_ticks = self.period_ticks[channel.index()];
        self.configure(channel, period_ticks, duty_ticks);
    }

    pub fn set_duty_fraction(&mut self, channel: Channel, numerator: u16, denominator: u16) {
        assert!(denominator != 0, "denominator must not be zero");
        let period_ticks = u32::from(self.period_ticks[channel.index()]);
        let duty_ticks = ((period_ticks * u32::from(numerator)) / u32::from(denominator)) as u16;
        self.set_duty_ticks(channel, duty_ticks);
    }

    pub fn set_duty_8bit(&mut self, channel: Channel, level: u8) {
        self.set_duty_fraction(channel, u16::from(level), 255);
    }

    pub fn set_inverted(&mut self, channel: Channel, inverted: bool) {
        self._inner.pwm_invert().modify(|r, w| unsafe {
            let mut bits = r.bits();
            if inverted {
                bits |= channel.bit();
            } else {
                bits &= !channel.bit();
            }
            w.bits(bits)
        });
    }

    pub fn set_polarity_active_high(&mut self, channel: Channel, active_high: bool) {
        self._inner.pwm_pol().modify(|r, w| unsafe {
            let mut bits = r.bits();
            if active_high {
                bits |= channel.bit();
            } else {
                bits &= !channel.bit();
            }
            w.bits(bits)
        });
    }

    pub fn period_ticks(&self, channel: Channel) -> u16 {
        self.period_ticks[channel.index()]
    }

    pub fn split(self) -> Channels {
        Channels {
            pwm0: PwmChannel::new(Channel::Pwm0),
            pwm1: PwmChannel::new(Channel::Pwm1),
            pwm2: PwmChannel::new(Channel::Pwm2),
            pwm3: PwmChannel::new(Channel::Pwm3),
            pwm4: PwmChannel::new(Channel::Pwm4),
            pwm5: PwmChannel::new(Channel::Pwm5),
        }
    }
}

impl PwmChannel {
    #[inline(always)]
    const fn new(channel: Channel) -> Self {
        Self {
            channel,
            period_ticks: 0,
        }
    }

    #[inline(always)]
    fn cycle_reg(&self) -> *mut u32 {
        (PWM_CYCLE_BASE + self.channel.index() * 4) as *mut u32
    }

    pub fn configure(&mut self, cycle_ticks: u16, duty_ticks: u16) {
        let duty_ticks = duty_ticks.min(cycle_ticks);
        self.period_ticks = cycle_ticks;
        if self.channel == Channel::Pwm0 {
            unsafe {
                core::ptr::write_volatile(PWM0_MODE_REG as *mut u8, 0);
            }
        }
        unsafe {
            core::ptr::write_volatile(
                self.cycle_reg(),
                u32::from(duty_ticks) | (u32::from(cycle_ticks) << 16),
            );
        }
    }

    pub fn enable(&mut self) {
        unsafe {
            match self.channel {
                Channel::Pwm0 => {
                    let reg = PWM0_ENABLE_REG as *mut u8;
                    let bits = core::ptr::read_volatile(reg.cast_const()) | 0x01;
                    core::ptr::write_volatile(reg, bits);
                }
                _ => {
                    let reg = PWM_ENABLE_REG as *mut u8;
                    let bits = core::ptr::read_volatile(reg.cast_const()) | self.channel.bit();
                    core::ptr::write_volatile(reg, bits);
                }
            }
        }
    }

    pub fn disable(&mut self) {
        unsafe {
            match self.channel {
                Channel::Pwm0 => {
                    let reg = PWM0_ENABLE_REG as *mut u8;
                    let bits = core::ptr::read_volatile(reg.cast_const()) & !0x01;
                    core::ptr::write_volatile(reg, bits);
                }
                _ => {
                    let reg = PWM_ENABLE_REG as *mut u8;
                    let bits = core::ptr::read_volatile(reg.cast_const()) & !self.channel.bit();
                    core::ptr::write_volatile(reg, bits);
                }
            }
        }
    }

    pub fn set_inverted(&mut self, inverted: bool) {
        unsafe {
            let reg = PWM_INVERT_REG as *mut u8;
            let mut bits = core::ptr::read_volatile(reg.cast_const());
            if inverted {
                bits |= self.channel.bit();
            } else {
                bits &= !self.channel.bit();
            }
            core::ptr::write_volatile(reg, bits);
        }
    }

    pub fn set_polarity_active_high(&mut self, active_high: bool) {
        unsafe {
            let reg = PWM_POL_REG as *mut u8;
            let mut bits = core::ptr::read_volatile(reg.cast_const());
            if active_high {
                bits |= self.channel.bit();
            } else {
                bits &= !self.channel.bit();
            }
            core::ptr::write_volatile(reg, bits);
        }
    }

    pub fn period_ticks(&self) -> u16 {
        self.period_ticks
    }

    pub fn set_duty_ticks(&mut self, duty_ticks: u16) {
        self.configure(self.period_ticks, duty_ticks);
    }

    pub fn set_duty_fraction(&mut self, numerator: u16, denominator: u16) {
        assert!(denominator != 0, "denominator must not be zero");
        let period_ticks = u32::from(self.period_ticks);
        let duty_ticks = ((period_ticks * u32::from(numerator)) / u32::from(denominator)) as u16;
        self.set_duty_ticks(duty_ticks);
    }

    pub fn set_duty_8bit(&mut self, level: u8) {
        self.set_duty_fraction(u16::from(level), 255);
    }
}

impl ErrorType for PwmChannel {
    type Error = Infallible;
}

impl SetDutyCycle for PwmChannel {
    fn max_duty_cycle(&self) -> u16 {
        self.period_ticks
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.set_duty_ticks(duty);
        Ok(())
    }
}
