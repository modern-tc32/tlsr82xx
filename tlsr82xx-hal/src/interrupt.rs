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
#[cfg(feature = "chip-8258")]
pub type RfIrqHandler = unsafe extern "C" fn(u16);
pub type VoidHandler = unsafe extern "C" fn();

unsafe extern "C" {
    static __ram_code_start: u8;
    static __ram_code_end: u8;
}

#[derive(Clone, Copy)]
pub struct RamIrqHandler(IrqHandler);

#[derive(Clone, Copy)]
pub struct RamGlobalIrqHandler(VoidHandler);

#[cfg(feature = "chip-8258")]
#[derive(Clone, Copy)]
pub struct RamRfIrqHandler(RfIrqHandler);

#[derive(Clone, Copy)]
pub struct RamVoidHandler(VoidHandler);

impl RamIrqHandler {
    #[doc(hidden)]
    pub const unsafe fn __new(handler: IrqHandler) -> Self {
        Self(handler)
    }

    #[inline(always)]
    const fn get(self) -> IrqHandler {
        self.0
    }
}

impl RamGlobalIrqHandler {
    #[doc(hidden)]
    pub const unsafe fn __new(handler: VoidHandler) -> Self {
        Self(handler)
    }

    #[inline(always)]
    const fn get(self) -> VoidHandler {
        self.0
    }
}

#[cfg(feature = "chip-8258")]
impl RamRfIrqHandler {
    #[doc(hidden)]
    pub const unsafe fn __new(handler: RfIrqHandler) -> Self {
        Self(handler)
    }

    #[inline(always)]
    const fn get(self) -> RfIrqHandler {
        self.0
    }
}

impl RamVoidHandler {
    #[doc(hidden)]
    pub const unsafe fn __new(handler: VoidHandler) -> Self {
        Self(handler)
    }

    #[inline(always)]
    pub(crate) const fn get(self) -> VoidHandler {
        self.0
    }
}

static mut IRQ_HANDLERS: [Option<IrqHandler>; 32] = [None; 32];
static mut GLOBAL_IRQ_HANDLER: Option<unsafe extern "C" fn()> = None;
#[cfg(feature = "chip-8258")]
static mut RF_IRQ_HANDLERS: [Option<RfIrqHandler>; 16] = [None; 16];

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

    #[inline(always)]
    pub const fn from_bit(bit: u8) -> Option<Self> {
        match bit {
            0 => Some(Self::Timer0),
            1 => Some(Self::Timer1),
            2 => Some(Self::Timer2),
            3 => Some(Self::UsbPwrdn),
            4 => Some(Self::Dma),
            5 => Some(Self::DmaFifo),
            6 => Some(Self::Uart),
            7 => Some(Self::MixCmd),
            8 => Some(Self::Ep0Setup),
            9 => Some(Self::Ep0Data),
            10 => Some(Self::Ep0Status),
            11 => Some(Self::SetInterface),
            12 => Some(Self::EndpointData),
            13 => Some(Self::ZigbeeRadio),
            14 => Some(Self::SoftwarePwm),
            16 => Some(Self::Usb250us),
            17 => Some(Self::UsbReset),
            18 => Some(Self::Gpio),
            19 => Some(Self::PowerManagement),
            20 => Some(Self::SystemTimer),
            21 => Some(Self::GpioRisc0),
            22 => Some(Self::GpioRisc1),
            _ => None,
        }
    }
}

#[cfg(all(feature = "chip-8258", not(feature = "custom-irq-handler")))]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".ram_code")]
pub extern "C" fn irq_handler_rust() {
    let global = unsafe { GLOBAL_IRQ_HANDLER };
    if let Some(handler) = global {
        unsafe {
            handler();
        }
        return;
    }
    dispatch_pending_8258(snapshot_pending_8258());
}

#[cfg(all(not(feature = "chip-8258"), not(feature = "custom-irq-handler")))]
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
pub fn register_irq_handler(irq: Irq, handler: RamIrqHandler) {
    register_handler(irq.bit(), handler);
}

#[inline(always)]
pub fn unregister_irq_handler(irq: Irq) {
    unregister_handler(irq.bit());
}

pub fn register_handler(bit: u8, handler: RamIrqHandler) {
    debug_assert!(bit < 32);
    let irq_enabled = disable();
    assert_ram_code_addr(handler.get() as usize, "irq handler");
    unsafe {
        IRQ_HANDLERS[bit as usize] = Some(handler.get());
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
        #[cfg(feature = "chip-8258")]
        {
            let rf_base = core::ptr::addr_of_mut!(RF_IRQ_HANDLERS).cast::<Option<RfIrqHandler>>();
            for index in 0..16 {
                core::ptr::write(rf_base.add(index), None);
            }
        }
    }
    restore(irq_enabled);
}

pub fn register_global_irq_handler(handler: RamGlobalIrqHandler) {
    let irq_enabled = disable();
    assert_ram_code_addr(handler.get() as usize, "global irq handler");
    unsafe {
        GLOBAL_IRQ_HANDLER = Some(handler.get());
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

#[cfg(feature = "chip-8258")]
#[unsafe(link_section = ".ram_code")]
pub fn dispatch_pending_8258(pending: Pending8258) {
    let mut core_pending = pending.core;
    while core_pending != 0 {
        let bit = core_pending.trailing_zeros() as u8;
        core_pending &= !(1u32 << u32::from(bit));

        let Some(irq) = Irq::from_bit(bit) else {
            clear_irq_source(1u32 << u32::from(bit));
            continue;
        };

        match irq {
            Irq::Timer0 => {
                dispatch_one(bit);
            }
            Irq::SystemTimer => {
                dispatch_one(bit);
            }
            Irq::ZigbeeRadio => {
                let rf_pending = masked_rf_irq_source();
                dispatch_rf_pending(rf_pending);
                acknowledge_irq(Irq::ZigbeeRadio);
                dispatch_one(bit);
            }
            _ => {
                acknowledge_irq(irq);
                dispatch_one(bit);
            }
        }
    }
}

#[cfg(feature = "chip-8258")]
#[unsafe(link_section = ".ram_code")]
fn dispatch_rf_pending(mut pending: u16) {
    while pending != 0 {
        let bit = pending.trailing_zeros() as usize;
        let mask = 1u16 << bit;
        pending &= !mask;
        let handler = unsafe { RF_IRQ_HANDLERS[bit] };
        if let Some(handler) = handler {
            unsafe {
                handler(mask);
            }
        }
        acknowledge_rf_irq(mask);
    }
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
fn dispatch_one(bit: u8) {
    let handler = unsafe { IRQ_HANDLERS[bit as usize] };
    if let Some(handler) = handler {
        unsafe {
            handler(1u32 << u32::from(bit));
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
pub fn register_rf_irq_handler(mask: u16, handler: RamRfIrqHandler) {
    let irq_enabled = disable();
    assert_ram_code_addr(handler.get() as usize, "rf irq handler");
    unsafe {
        let base = core::ptr::addr_of_mut!(RF_IRQ_HANDLERS).cast::<Option<RfIrqHandler>>();
        let mut pending = mask;
        while pending != 0 {
            let bit = pending.trailing_zeros() as usize;
            pending &= !(1u16 << bit);
            if bit < 16 {
                core::ptr::write(base.add(bit), Some(handler.get()));
            }
        }
    }
    restore(irq_enabled);
}

#[cfg(feature = "chip-8258")]
pub fn unregister_rf_irq_handler(mask: u16) {
    let irq_enabled = disable();
    unsafe {
        let base = core::ptr::addr_of_mut!(RF_IRQ_HANDLERS).cast::<Option<RfIrqHandler>>();
        let mut pending = mask;
        while pending != 0 {
            let bit = pending.trailing_zeros() as usize;
            pending &= !(1u16 << bit);
            if bit < 16 {
                core::ptr::write(base.add(bit), None);
            }
        }
    }
    restore(irq_enabled);
}

#[cfg(feature = "chip-8258")]
#[inline(always)]
pub fn snapshot_pending_8258() -> Pending8258 {
    Pending8258 {
        core: masked_irq_source(),
        rf: masked_rf_irq_source(),
    }
}

pub(crate) fn assert_ram_code_addr(addr: usize, kind: &str) {
    let start = core::ptr::addr_of!(__ram_code_start) as usize;
    let end = core::ptr::addr_of!(__ram_code_end) as usize;
    assert!(
        addr >= start && addr < end,
        "{kind} 0x{addr:08x} not in .ram_code [0x{start:08x}, 0x{end:08x})"
    );
}
