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

unsafe extern "C" {
    fn clock_init(sys_clk: u8);
    static system_clk_type: u8;
    static system_clk_mHz: u8;
}

#[inline(always)]
pub fn init(clock: SysClock) {
    unsafe {
        clock_init(clock as u8);
    }
}

#[inline(always)]
pub fn current() -> u8 {
    unsafe { system_clk_type }
}

#[inline(always)]
pub fn current_mhz() -> u8 {
    unsafe { system_clk_mHz }
}
