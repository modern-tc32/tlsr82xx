use crate::mmio::{reg16, reg32, reg8};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{REG_IRQ_EN, REG_IRQ_MASK, REG_IRQ_SRC, REG_RF_IRQ_MASK, REG_RF_IRQ_STATUS};

pub const ALL_IRQS: u32 = 0xffff_ffff;

#[unsafe(no_mangle)]
pub extern "C" fn irq_handler() {}

#[inline(always)]
pub fn enable() -> bool {
    unsafe {
        let reg = reg8(REG_IRQ_EN);
        let prev = core::ptr::read_volatile(reg);
        core::ptr::write_volatile(reg, 1);
        prev != 0
    }
}

#[inline(always)]
pub fn disable() -> bool {
    unsafe {
        let reg = reg8(REG_IRQ_EN);
        let prev = core::ptr::read_volatile(reg);
        core::ptr::write_volatile(reg, 0);
        prev != 0
    }
}

#[inline(always)]
pub fn restore(enabled: bool) {
    unsafe {
        core::ptr::write_volatile(reg8(REG_IRQ_EN), enabled as u8);
    }
}

#[inline(always)]
pub fn mask() -> u32 {
    unsafe { core::ptr::read_volatile(reg32(REG_IRQ_MASK).cast_const()) }
}

#[inline(always)]
pub fn set_mask(mask: u32) {
    unsafe {
        let reg = reg32(REG_IRQ_MASK);
        core::ptr::write_volatile(reg, core::ptr::read_volatile(reg.cast_const()) | mask);
    }
}

#[inline(always)]
pub fn clear_mask(mask: u32) {
    unsafe {
        let reg = reg32(REG_IRQ_MASK);
        core::ptr::write_volatile(reg, core::ptr::read_volatile(reg.cast_const()) & !mask);
    }
}

#[inline(always)]
pub fn irq_source() -> u32 {
    unsafe { core::ptr::read_volatile(reg32(REG_IRQ_SRC).cast_const()) }
}

#[inline(always)]
pub fn clear_irq_source(mask: u32) {
    unsafe {
        core::ptr::write_volatile(reg32(REG_IRQ_SRC), mask);
    }
}

#[inline(always)]
pub fn clear_all_irq_sources() {
    clear_irq_source(ALL_IRQS);
}

#[inline(always)]
pub fn rf_set_mask(mask: u16) {
    unsafe {
        let reg = reg16(REG_RF_IRQ_MASK);
        core::ptr::write_volatile(reg, core::ptr::read_volatile(reg.cast_const()) | mask);
    }
}

#[inline(always)]
pub fn rf_clear_mask(mask: u16) {
    unsafe {
        let reg = reg16(REG_RF_IRQ_MASK);
        core::ptr::write_volatile(reg, core::ptr::read_volatile(reg.cast_const()) & !mask);
    }
}

#[inline(always)]
pub fn rf_irq_source() -> u16 {
    unsafe { core::ptr::read_volatile(reg16(REG_RF_IRQ_STATUS).cast_const()) }
}

#[inline(always)]
pub fn rf_clear_irq_source(mask: u16) {
    unsafe {
        core::ptr::write_volatile(reg16(REG_RF_IRQ_STATUS), mask);
    }
}
