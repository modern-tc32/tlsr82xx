use crate::analog;

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

const CLK_SEL_ADDR: usize = 0x0080_0066;

#[inline(always)]
fn reg8(addr: usize) -> *mut u8 {
    addr as *mut u8
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
        core::ptr::write_volatile(reg8(CLK_SEL_ADDR), sys_clk);
        system_clk_type = sys_clk;
        system_clk_mHz = mhz;
    }

    if sys_clk == SysClock::Crystal48M as u8 {
        analog::write(0x0c, 0xc6);
    }
}

#[inline(always)]
pub fn init(clock: SysClock) {
    clock_init(clock as u8);
}

#[inline(always)]
pub fn current() -> u8 {
    unsafe { system_clk_type }
}

#[inline(always)]
pub fn current_mhz() -> u8 {
    unsafe { system_clk_mHz }
}
