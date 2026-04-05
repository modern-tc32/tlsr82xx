const IRQ_MASK_ADDR: usize = 0x0080_0640;
const IRQ_EN_ADDR: usize = 0x0080_0643;
const IRQ_SRC_ADDR: usize = 0x0080_0648;
const RF_IRQ_MASK_ADDR: usize = 0x0080_0f1c;
const RF_IRQ_STATUS_ADDR: usize = 0x0080_0f20;

pub const ALL_IRQS: u32 = 0xffff_ffff;

#[inline(always)]
fn reg8(addr: usize) -> *mut u8 {
    addr as *mut u8
}

#[inline(always)]
fn reg16(addr: usize) -> *mut u16 {
    addr as *mut u16
}

#[inline(always)]
fn reg32(addr: usize) -> *mut u32 {
    addr as *mut u32
}

#[inline(always)]
pub fn enable() -> bool {
    unsafe {
        let reg = reg8(IRQ_EN_ADDR);
        let prev = core::ptr::read_volatile(reg);
        core::ptr::write_volatile(reg, 1);
        prev != 0
    }
}

#[inline(always)]
pub fn disable() -> bool {
    unsafe {
        let reg = reg8(IRQ_EN_ADDR);
        let prev = core::ptr::read_volatile(reg);
        core::ptr::write_volatile(reg, 0);
        prev != 0
    }
}

#[inline(always)]
pub fn restore(enabled: bool) {
    unsafe {
        core::ptr::write_volatile(reg8(IRQ_EN_ADDR), enabled as u8);
    }
}

#[inline(always)]
pub fn mask() -> u32 {
    unsafe { core::ptr::read_volatile(reg32(IRQ_MASK_ADDR).cast_const()) }
}

#[inline(always)]
pub fn set_mask(mask: u32) {
    unsafe {
        let reg = reg32(IRQ_MASK_ADDR);
        core::ptr::write_volatile(reg, core::ptr::read_volatile(reg.cast_const()) | mask);
    }
}

#[inline(always)]
pub fn clear_mask(mask: u32) {
    unsafe {
        let reg = reg32(IRQ_MASK_ADDR);
        core::ptr::write_volatile(reg, core::ptr::read_volatile(reg.cast_const()) & !mask);
    }
}

#[inline(always)]
pub fn irq_source() -> u32 {
    unsafe { core::ptr::read_volatile(reg32(IRQ_SRC_ADDR).cast_const()) }
}

#[inline(always)]
pub fn clear_irq_source(mask: u32) {
    unsafe {
        core::ptr::write_volatile(reg32(IRQ_SRC_ADDR), mask);
    }
}

#[inline(always)]
pub fn clear_all_irq_sources() {
    clear_irq_source(ALL_IRQS);
}

#[inline(always)]
pub fn rf_set_mask(mask: u16) {
    unsafe {
        let reg = reg16(RF_IRQ_MASK_ADDR);
        core::ptr::write_volatile(reg, core::ptr::read_volatile(reg.cast_const()) | mask);
    }
}

#[inline(always)]
pub fn rf_clear_mask(mask: u16) {
    unsafe {
        let reg = reg16(RF_IRQ_MASK_ADDR);
        core::ptr::write_volatile(reg, core::ptr::read_volatile(reg.cast_const()) & !mask);
    }
}

#[inline(always)]
pub fn rf_irq_source() -> u16 {
    unsafe { core::ptr::read_volatile(reg16(RF_IRQ_STATUS_ADDR).cast_const()) }
}

#[inline(always)]
pub fn rf_clear_irq_source(mask: u16) {
    unsafe {
        core::ptr::write_volatile(reg16(RF_IRQ_STATUS_ADDR), mask);
    }
}
