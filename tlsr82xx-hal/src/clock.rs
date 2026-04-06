use crate::analog;
use crate::mmio::reg8;
#[cfg(feature = "chip-8258")]
use crate::regs8258::{AREG_FLASH_VOLTAGE, REG_CLK_SEL};

unsafe extern "C" {
    static mut tl_24mrc_cal: u8;
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SysClock {
    Crystal12M = 0x44,
    Crystal16M = 0x43,
    Crystal24M = 0x42,
    Crystal32M = 0x60,
    Crystal48M = 0x20,
    Rc24M = 0x00,
    Rc32M = 0x01,
    Rc48M = 0x02,
}

impl SysClock {
    #[inline(always)]
    pub const fn mhz(self) -> u8 {
        match self {
            Self::Crystal12M => 12,
            Self::Crystal16M => 16,
            Self::Crystal24M | Self::Rc24M => 24,
            Self::Crystal32M | Self::Rc32M => 32,
            Self::Crystal48M | Self::Rc48M => 48,
        }
    }
}

#[unsafe(no_mangle)]
pub static mut system_clk_type: u8 = SysClock::Crystal48M as u8;

#[unsafe(no_mangle)]
pub static mut system_clk_mHz: u8 = 48;

#[unsafe(no_mangle)]
pub extern "C" fn clock_init(sys_clk: u8) {
    let mhz = match sys_clk {
        x if x == SysClock::Crystal12M as u8 => 12,
        x if x == SysClock::Crystal16M as u8 => 16,
        x if x == SysClock::Crystal24M as u8 || x == SysClock::Rc24M as u8 => 24,
        x if x == SysClock::Crystal32M as u8 || x == SysClock::Rc32M as u8 => 32,
        x if x == SysClock::Crystal48M as u8 || x == SysClock::Rc48M as u8 => 48,
        _ => return,
    };

    unsafe {
        core::ptr::write_volatile(reg8(REG_CLK_SEL), sys_clk);
        system_clk_type = sys_clk;
        system_clk_mHz = mhz;
    }

    if sys_clk == SysClock::Crystal48M as u8 {
        analog::write(AREG_FLASH_VOLTAGE, 0xc6);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rc_24m_cal() {
    analog::write(0xc8, 0x80);

    let clk = analog::read(0x30) | 0x80;
    analog::write(0x30, clk);

    analog::write(0xc7, 0x0e);
    analog::write(0xc7, 0x0f);

    while (analog::read(0xcf) & 0x80) == 0 {}

    let cal = analog::read(0xcb);
    analog::write(0x33, cal);

    let clk = analog::read(0x30) & 0x7f;
    analog::write(0x30, clk);
    analog::write(0xc7, 0x0e);

    unsafe {
        tl_24mrc_cal = analog::read(0x33);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn doubler_calibration() {
    analog::write(0x86, 0xbb);

    let val = analog::read(0x82) & 0x7f;
    analog::write(0x82, val);

    let val = (analog::read(0x87) & !0x03) | 0x02;
    analog::write(0x87, val);

    let val = analog::read(0x87) | 0x04;
    analog::write(0x87, val);

    let high = analog::read(0x88) & 0x1f;
    let low = analog::read(0x87) & 0x07;
    analog::write(0x87, (high << 3) | low);

    let val = analog::read(0x82) | 0x80;
    analog::write(0x82, val);

    let val = analog::read(0x87) & 0xfd;
    analog::write(0x87, val);

    analog::write(0x86, 0xfb);
}

#[inline(always)]
pub fn init(clock: SysClock) {
    clock_init(clock as u8);
}

#[inline(always)]
pub fn current() -> u8 {
    unsafe {
        unsafe extern "C" {
            static system_clk_type: u8;
        }
        system_clk_type
    }
}

#[inline(always)]
pub fn current_mhz() -> u8 {
    unsafe {
        unsafe extern "C" {
            static system_clk_mHz: u8;
        }
        system_clk_mHz
    }
}
