use crate::mmio::{reg16, reg32, reg8};
#[cfg(feature = "chip-8258")]
use crate::regs8258::{
    FLD_IRQ_GPIO_EN, FLD_IRQ_GPIO_RISC0_EN, FLD_IRQ_GPIO_RISC1_EN, FLD_IRQ_SYSTEM_TIMER,
    FLD_IRQ_TMR0_EN, FLD_IRQ_TMR1_EN, FLD_IRQ_TMR2_EN, FLD_TMR_STA_TMR0, FLD_TMR_STA_TMR1,
    FLD_TMR_STA_TMR2, REG_IRQ_EN, REG_IRQ_MASK, REG_IRQ_SRC, REG_RF_IRQ_MASK, REG_RF_IRQ_STATUS,
    REG_TMR_STA,
};

pub const ALL_IRQS: u32 = 0xffff_ffff;

pub type IrqHandler = unsafe extern "C" fn(u32);

static mut IRQ_HANDLERS: [Option<IrqHandler>; 32] = [None; 32];
static mut GLOBAL_IRQ_HANDLER: Option<unsafe extern "C" fn()> = None;

#[cfg(feature = "chip-8258")]
#[derive(Clone, Copy, Debug, Default)]
pub struct Pending8258 {
    pub core: u32,
    pub rf: u16,
}

#[cfg(feature = "chip-8258")]
impl Pending8258 {
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.core == 0 && self.rf == 0
    }

    #[inline(always)]
    pub const fn has_irq(self, irq: Irq) -> bool {
        (self.core & irq.mask()) != 0
    }

    #[inline(always)]
    pub const fn has_rf(self, mask: u16) -> bool {
        (self.rf & mask) != 0
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Irq {
    Timer0 = 0,
    Timer1 = 1,
    Timer2 = 2,
    UsbPwrdn = 3,
    Dma = 4,
    DmaFifo = 5,
    Uart = 6,
    MixCmd = 7,
    Ep0Setup = 8,
    Ep0Data = 9,
    Ep0Status = 10,
    SetInterface = 11,
    EndpointData = 12,
    ZigbeeRadio = 13,
    SoftwarePwm = 14,
    Usb250us = 16,
    UsbReset = 17,
    Gpio = 18,
    PowerManagement = 19,
    SystemTimer = 20,
    GpioRisc0 = 21,
    GpioRisc1 = 22,
}

impl Irq {
    #[inline(always)]
    pub const fn bit(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub const fn mask(self) -> u32 {
        1u32 << (self as u32)
    }
}

#[cfg(not(feature = "custom-irq-handler"))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code")]
pub extern "C" fn irq_handler() {
    let global = unsafe { GLOBAL_IRQ_HANDLER };
    if let Some(handler) = global {
        unsafe {
            handler();
        }
        return;
    }
    let pending = irq_source() & mask();
    dispatch_pending(pending);
}

#[inline(always)]
pub fn enable_irq(irq: Irq) {
    set_mask(irq.mask());
}

#[inline(always)]
pub fn disable_irq(irq: Irq) {
    clear_mask(irq.mask());
}

#[inline(always)]
pub fn clear_irq(irq: Irq) {
    clear_irq_source(irq.mask());
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn acknowledge_irq(irq: Irq) {
    unsafe {
        match irq {
            Irq::Timer0 => {
                core::ptr::write_volatile(reg32(REG_IRQ_SRC), FLD_IRQ_TMR0_EN);
                core::ptr::write_volatile(reg8(REG_TMR_STA), FLD_TMR_STA_TMR0);
            }
            Irq::Timer1 => {
                core::ptr::write_volatile(reg32(REG_IRQ_SRC), FLD_IRQ_TMR1_EN);
                core::ptr::write_volatile(reg8(REG_TMR_STA), FLD_TMR_STA_TMR1);
            }
            Irq::Timer2 => {
                core::ptr::write_volatile(reg32(REG_IRQ_SRC), FLD_IRQ_TMR2_EN);
                core::ptr::write_volatile(reg8(REG_TMR_STA), FLD_TMR_STA_TMR2);
            }
            Irq::SystemTimer => {
                core::ptr::write_volatile(reg32(REG_IRQ_SRC), FLD_IRQ_SYSTEM_TIMER);
            }
            Irq::Gpio => {
                core::ptr::write_volatile(reg32(REG_IRQ_SRC), FLD_IRQ_GPIO_EN);
            }
            Irq::GpioRisc0 => {
                core::ptr::write_volatile(reg32(REG_IRQ_SRC), FLD_IRQ_GPIO_RISC0_EN);
            }
            Irq::GpioRisc1 => {
                core::ptr::write_volatile(reg32(REG_IRQ_SRC), FLD_IRQ_GPIO_RISC1_EN);
            }
            _ => {
                core::ptr::write_volatile(reg32(REG_IRQ_SRC), irq.mask());
            }
        }
    }
}

#[cfg(not(feature = "chip-8258"))]
#[inline(always)]
pub fn acknowledge_irq(irq: Irq) {
    clear_irq(irq);
}

#[inline(always)]
pub fn is_pending(irq: Irq) -> bool {
    (irq_source() & irq.mask()) != 0
}

#[inline(always)]
pub fn register_irq_handler(irq: Irq, handler: IrqHandler) {
    register_handler(irq.bit(), handler);
}

#[inline(always)]
pub fn unregister_irq_handler(irq: Irq) {
    unregister_handler(irq.bit());
}

pub fn register_handler(bit: u8, handler: IrqHandler) {
    debug_assert!(bit < 32);
    let irq_enabled = disable();
    unsafe {
        IRQ_HANDLERS[bit as usize] = Some(handler);
    }
    restore(irq_enabled);
}

pub fn unregister_handler(bit: u8) {
    debug_assert!(bit < 32);
    let irq_enabled = disable();
    unsafe {
        IRQ_HANDLERS[bit as usize] = None;
    }
    restore(irq_enabled);
}

pub fn clear_handlers() {
    let irq_enabled = disable();
    unsafe {
        let base = core::ptr::addr_of_mut!(IRQ_HANDLERS).cast::<Option<IrqHandler>>();
        for index in 0..32 {
            core::ptr::write(base.add(index), None);
        }
    }
    restore(irq_enabled);
}

pub fn register_global_irq_handler(handler: unsafe extern "C" fn()) {
    let irq_enabled = disable();
    unsafe {
        GLOBAL_IRQ_HANDLER = Some(handler);
    }
    restore(irq_enabled);
}

pub fn unregister_global_irq_handler() {
    let irq_enabled = disable();
    unsafe {
        GLOBAL_IRQ_HANDLER = None;
    }
    restore(irq_enabled);
}

#[unsafe(link_section = ".ram_code")]
pub fn dispatch_pending(mut pending: u32) {
    while pending != 0 {
        let bit = pending.trailing_zeros() as usize;
        pending &= !(1u32 << bit);
        let handler = unsafe { IRQ_HANDLERS[bit] };
        if let Some(handler) = handler {
            unsafe {
                handler(1u32 << bit);
            }
        }
    }
}

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
pub fn masked_irq_source() -> u32 {
    irq_source() & mask()
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
pub fn rf_mask() -> u16 {
    unsafe { core::ptr::read_volatile(reg16(REG_RF_IRQ_MASK).cast_const()) }
}

#[inline(always)]
pub fn masked_rf_irq_source() -> u16 {
    rf_irq_source() & rf_mask()
}

#[inline(always)]
pub fn rf_clear_irq_source(mask: u16) {
    unsafe {
        core::ptr::write_volatile(reg16(REG_RF_IRQ_STATUS), mask);
    }
}

#[inline(always)]
pub fn acknowledge_rf_irq(mask: u16) {
    rf_clear_irq_source(mask);
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn snapshot_pending_8258() -> Pending8258 {
    Pending8258 {
        core: masked_irq_source(),
        rf: masked_rf_irq_source(),
    }
}
