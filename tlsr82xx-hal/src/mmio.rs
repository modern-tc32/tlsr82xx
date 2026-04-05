#[inline(always)]
pub(crate) fn reg8(addr: usize) -> *mut u8 {
    addr as *mut u8
}

#[inline(always)]
pub(crate) fn reg16(addr: usize) -> *mut u16 {
    addr as *mut u16
}

#[inline(always)]
pub(crate) fn reg32(addr: usize) -> *mut u32 {
    addr as *mut u32
}
