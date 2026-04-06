const ANALOG_CTRL_BASE: usize = 0x0080_00B8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pull {
    Floating,
    PullUp1M,
    PullDown100K,
    PullUp10K,
}

impl Pull {
    #[inline(always)]
    pub(crate) const fn bits(self) -> u8 {
        match self {
            Self::Floating => 0,
            Self::PullUp1M => 1,
            Self::PullDown100K => 2,
            Self::PullUp10K => 3,
        }
    }
}

#[inline(always)]
pub fn read(addr: u8) -> u8 {
    let reg = ANALOG_CTRL_BASE as *mut u8;
    unsafe {
        core::ptr::write_volatile(reg.add(0), addr);
        core::ptr::write_volatile(reg.add(2), 0x40);
        while (core::ptr::read_volatile(reg.add(2)) & 1) != 0 {}
        core::ptr::read_volatile(reg.add(1))
    }
}

#[inline(always)]
pub fn write(addr: u8, value: u8) {
    let reg = ANALOG_CTRL_BASE as *mut u8;
    unsafe {
        core::ptr::write_volatile(reg.add(0), addr);
        core::ptr::write_volatile(reg.add(1), value);
        core::ptr::write_volatile(reg.add(2), 0x60);
        while (core::ptr::read_volatile(reg.add(2)) & 1) != 0 {}
        core::ptr::write_volatile(reg.add(2), 0);
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.analog_read")]
pub extern "C" fn analog_read(addr: u8) -> u8 {
    read(addr)
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code.analog_write")]
pub extern "C" fn analog_write(addr: u8, value: u8) {
    write(addr, value)
}
