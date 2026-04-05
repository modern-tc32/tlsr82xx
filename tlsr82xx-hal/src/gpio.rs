use core::convert::Infallible;
use core::marker::PhantomData;

use embedded_hal::digital::{
    ErrorType, InputPin as EhInputPin, OutputPin as EhOutputPin,
    StatefulOutputPin as EhStatefulOutputPin,
};

use crate::{analog, pac};

const PORT_A: u8 = 0;
const PORT_B: u8 = 1;
const PORT_C: u8 = 2;
const PORT_D: u8 = 3;
const PORT_E: u8 = 4;

const GPIO_BASE: usize = 0x0080_0580;
const IRQ_BASE: usize = 0x0080_0640;

pub struct Input;
pub struct Output;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Level {
    Low,
    High,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriveStrength {
    Weak,
    Strong,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptEdge {
    Rising,
    Falling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptRoute {
    Core,
    Risc0,
    Risc1,
}

pub struct Pin<const PORT: u8, const BIT: u8, MODE> {
    _mode: PhantomData<MODE>,
}

pub type PA0<MODE = Input> = Pin<PORT_A, 0, MODE>;
pub type PA1<MODE = Input> = Pin<PORT_A, 1, MODE>;
pub type PA2<MODE = Input> = Pin<PORT_A, 2, MODE>;
pub type PA3<MODE = Input> = Pin<PORT_A, 3, MODE>;
pub type PA4<MODE = Input> = Pin<PORT_A, 4, MODE>;
pub type PA5<MODE = Input> = Pin<PORT_A, 5, MODE>;
pub type PA6<MODE = Input> = Pin<PORT_A, 6, MODE>;
pub type PA7<MODE = Input> = Pin<PORT_A, 7, MODE>;

pub type PB0<MODE = Input> = Pin<PORT_B, 0, MODE>;
pub type PB1<MODE = Input> = Pin<PORT_B, 1, MODE>;
pub type PB2<MODE = Input> = Pin<PORT_B, 2, MODE>;
pub type PB3<MODE = Input> = Pin<PORT_B, 3, MODE>;
pub type PB4<MODE = Input> = Pin<PORT_B, 4, MODE>;
pub type PB5<MODE = Input> = Pin<PORT_B, 5, MODE>;
pub type PB6<MODE = Input> = Pin<PORT_B, 6, MODE>;
pub type PB7<MODE = Input> = Pin<PORT_B, 7, MODE>;

pub type PC0<MODE = Input> = Pin<PORT_C, 0, MODE>;
pub type PC1<MODE = Input> = Pin<PORT_C, 1, MODE>;
pub type PC2<MODE = Input> = Pin<PORT_C, 2, MODE>;
pub type PC3<MODE = Input> = Pin<PORT_C, 3, MODE>;
pub type PC4<MODE = Input> = Pin<PORT_C, 4, MODE>;
pub type PC5<MODE = Input> = Pin<PORT_C, 5, MODE>;
pub type PC6<MODE = Input> = Pin<PORT_C, 6, MODE>;
pub type PC7<MODE = Input> = Pin<PORT_C, 7, MODE>;

pub type PD0<MODE = Input> = Pin<PORT_D, 0, MODE>;
pub type PD1<MODE = Input> = Pin<PORT_D, 1, MODE>;
pub type PD2<MODE = Input> = Pin<PORT_D, 2, MODE>;
pub type PD3<MODE = Input> = Pin<PORT_D, 3, MODE>;
pub type PD4<MODE = Input> = Pin<PORT_D, 4, MODE>;
pub type PD5<MODE = Input> = Pin<PORT_D, 5, MODE>;
pub type PD6<MODE = Input> = Pin<PORT_D, 6, MODE>;
pub type PD7<MODE = Input> = Pin<PORT_D, 7, MODE>;

pub type PE0<MODE = Input> = Pin<PORT_E, 0, MODE>;
pub type PE1<MODE = Input> = Pin<PORT_E, 1, MODE>;
pub type PE2<MODE = Input> = Pin<PORT_E, 2, MODE>;
pub type PE3<MODE = Input> = Pin<PORT_E, 3, MODE>;

pub struct Pins {
    pub pa0: PA0,
    pub pa1: PA1,
    pub pa2: PA2,
    pub pa3: PA3,
    pub pa4: PA4,
    pub pa5: PA5,
    pub pa6: PA6,
    pub pa7: PA7,
    pub pb0: PB0,
    pub pb1: PB1,
    pub pb2: PB2,
    pub pb3: PB3,
    pub pb4: PB4,
    pub pb5: PB5,
    pub pb6: PB6,
    pub pb7: PB7,
    pub pc0: PC0,
    pub pc1: PC1,
    pub pc2: PC2,
    pub pc3: PC3,
    pub pc4: PC4,
    pub pc5: PC5,
    pub pc6: PC6,
    pub pc7: PC7,
    pub pd0: PD0,
    pub pd1: PD1,
    pub pd2: PD2,
    pub pd3: PD3,
    pub pd4: PD4,
    pub pd5: PD5,
    pub pd6: PD6,
    pub pd7: PD7,
    pub pe0: PE0,
    pub pe1: PE1,
    pub pe2: PE2,
    pub pe3: PE3,
}

pub trait GpioExt {
    fn split(self) -> Pins;
}

impl GpioExt for pac::Gpio {
    fn split(self) -> Pins {
        Pins::new(self)
    }
}

impl Pins {
    pub fn new(_gpio: pac::Gpio) -> Self {
        Self {
            pa0: Pin::new(),
            pa1: Pin::new(),
            pa2: Pin::new(),
            pa3: Pin::new(),
            pa4: Pin::new(),
            pa5: Pin::new(),
            pa6: Pin::new(),
            pa7: Pin::new(),
            pb0: Pin::new(),
            pb1: Pin::new(),
            pb2: Pin::new(),
            pb3: Pin::new(),
            pb4: Pin::new(),
            pb5: Pin::new(),
            pb6: Pin::new(),
            pb7: Pin::new(),
            pc0: Pin::new(),
            pc1: Pin::new(),
            pc2: Pin::new(),
            pc3: Pin::new(),
            pc4: Pin::new(),
            pc5: Pin::new(),
            pc6: Pin::new(),
            pc7: Pin::new(),
            pd0: Pin::new(),
            pd1: Pin::new(),
            pd2: Pin::new(),
            pd3: Pin::new(),
            pd4: Pin::new(),
            pd5: Pin::new(),
            pd6: Pin::new(),
            pd7: Pin::new(),
            pe0: Pin::new(),
            pe1: Pin::new(),
            pe2: Pin::new(),
            pe3: Pin::new(),
        }
    }
}

impl<const PORT: u8, const BIT: u8, MODE> Pin<PORT, BIT, MODE> {
    const fn new() -> Self {
        Self { _mode: PhantomData }
    }

    #[inline(always)]
    const fn mask() -> u8 {
        1u8 << BIT
    }

    #[inline(always)]
    pub const fn raw_pin() -> u16 {
        ((PORT as u16) << 8) | Self::mask() as u16
    }

    #[inline(always)]
    const fn reg(offset: usize) -> *mut u8 {
        (GPIO_BASE + ((PORT as usize) << 3) + offset) as *mut u8
    }

    #[inline(always)]
    const fn route_reg(route: InterruptRoute) -> *mut u8 {
        match route {
            InterruptRoute::Core => Self::reg(0x07),
            InterruptRoute::Risc0 => (GPIO_BASE + 0x38 + PORT as usize) as *mut u8,
            InterruptRoute::Risc1 => (GPIO_BASE + 0x40 + PORT as usize) as *mut u8,
        }
    }

    #[inline(always)]
    fn read_reg(offset: usize) -> u8 {
        unsafe { core::ptr::read_volatile(Self::reg(offset).cast_const()) }
    }

    #[inline(always)]
    fn modify_reg(offset: usize, set: bool) {
        unsafe {
            let reg = Self::reg(offset);
            let mut value = core::ptr::read_volatile(reg.cast_const());
            if set {
                value |= Self::mask();
            } else {
                value &= !Self::mask();
            }
            core::ptr::write_volatile(reg, value);
        }
    }

    #[inline(always)]
    fn modify_raw_reg(reg: *mut u8, mask: u8, set: bool) {
        unsafe {
            let mut value = core::ptr::read_volatile(reg.cast_const());
            if set {
                value |= mask;
            } else {
                value &= !mask;
            }
            core::ptr::write_volatile(reg, value);
        }
    }

    #[inline(always)]
    fn write_irq_mask(mask: u32) {
        let reg = IRQ_BASE as *mut u32;
        unsafe {
            let value = core::ptr::read_volatile(reg.cast_const()) | mask;
            core::ptr::write_volatile(reg, value);
        }
    }

    #[inline(always)]
    fn clear_irq_src(mask: u32) {
        unsafe {
            core::ptr::write_volatile((IRQ_BASE + 0x08) as *mut u32, mask);
        }
    }

    #[inline(always)]
    fn set_wakeup_irq_flag(mask: u8, enabled: bool) {
        Self::modify_raw_reg((GPIO_BASE + 0x35) as *mut u8, mask, enabled);
    }

    #[inline(always)]
    fn pull_addr_shift() -> Option<(u8, u8)> {
        #[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
        {
            if PORT <= PORT_D {
                let addr = 0x0e + PORT + (BIT / 4);
                let shift = (BIT % 4) * 2;
                return Some((addr, shift));
            }
            return None;
        }

        #[cfg(feature = "chip-826x")]
        {
            return match (PORT, BIT) {
                (PORT_A, 0) => Some((0x0a, 4)),
                (PORT_A, 1) => Some((0x0a, 6)),
                (PORT_A, 2) => Some((0x0b, 0)),
                (PORT_A, 3) => Some((0x0b, 2)),
                (PORT_A, 4) => Some((0x0b, 4)),
                (PORT_A, 5) => Some((0x0b, 6)),
                (PORT_A, 6) => Some((0x0c, 0)),
                (PORT_A, 7) => Some((0x0c, 2)),
                (PORT_B, 0) => Some((0x0c, 4)),
                (PORT_B, 1) => Some((0x0c, 6)),
                (PORT_B, 2) => Some((0x0d, 0)),
                (PORT_B, 3) => Some((0x0d, 2)),
                (PORT_B, 4) => Some((0x0d, 4)),
                (PORT_B, 5) => Some((0x0d, 6)),
                (PORT_B, 6) => Some((0x0e, 0)),
                (PORT_B, 7) => Some((0x0e, 2)),
                (PORT_C, 0) => Some((0x0e, 4)),
                (PORT_C, 1) => Some((0x0e, 6)),
                (PORT_C, 2) => Some((0x0f, 0)),
                (PORT_C, 3) => Some((0x0f, 2)),
                (PORT_C, 4) => Some((0x0f, 4)),
                (PORT_C, 5) => Some((0x0f, 6)),
                (PORT_C, 6) => Some((0x10, 0)),
                (PORT_C, 7) => Some((0x10, 2)),
                (PORT_D, 0) => Some((0x10, 4)),
                (PORT_D, 1) => Some((0x10, 6)),
                (PORT_D, 2) => Some((0x11, 0)),
                (PORT_D, 3) => Some((0x11, 2)),
                (PORT_D, 4) => Some((0x11, 4)),
                (PORT_D, 5) => Some((0x11, 6)),
                (PORT_D, 6) => Some((0x12, 0)),
                (PORT_D, 7) => Some((0x12, 2)),
                (PORT_E, 0) => Some((0x12, 4)),
                (PORT_E, 1) => Some((0x12, 6)),
                (PORT_E, 2) => Some((0x08, 4)),
                (PORT_E, 3) => Some((0x08, 6)),
                _ => None,
            };
        }

        #[allow(unreachable_code)]
        None
    }

    #[inline(always)]
    fn configure_as_gpio() {
        Self::modify_reg(0x06, true);
    }

    #[inline(always)]
    fn set_input_enabled(enabled: bool) {
        Self::modify_reg(0x01, enabled);
    }

    #[inline(always)]
    fn set_output_enabled(enabled: bool) {
        // OEN is active-low in the hardware register.
        Self::modify_reg(0x02, !enabled);
    }

    #[inline(always)]
    fn write_data(high: bool) {
        Self::modify_reg(0x03, high);
    }

    #[inline(always)]
    fn set_drive_strength_raw(strong: bool) {
        Self::modify_reg(0x05, strong);
    }

    #[inline(always)]
    fn set_interrupt_polarity(edge: InterruptEdge) {
        Self::modify_reg(0x04, matches!(edge, InterruptEdge::Falling));
    }

    #[inline(always)]
    fn read_input_level() -> bool {
        (Self::read_reg(0x00) & Self::mask()) != 0
    }

    #[inline(always)]
    fn read_output_level() -> bool {
        (Self::read_reg(0x03) & Self::mask()) != 0
    }

    pub fn into_input(self) -> Pin<PORT, BIT, Input> {
        Self::configure_as_gpio();
        Self::set_output_enabled(false);
        Self::set_input_enabled(true);
        Pin::new()
    }

    pub fn into_output(self) -> Pin<PORT, BIT, Output> {
        self.into_output_with_state(Level::Low)
    }

    pub fn into_output_with_state(self, initial: Level) -> Pin<PORT, BIT, Output> {
        Self::configure_as_gpio();
        Self::write_data(matches!(initial, Level::High));
        Self::set_input_enabled(false);
        Self::set_output_enabled(true);
        Pin::new()
    }

    pub fn set_drive_strength(&mut self, strength: DriveStrength) {
        Self::set_drive_strength_raw(matches!(strength, DriveStrength::Strong));
    }

    pub fn set_pull_resistor(&mut self, pull: analog::Pull) {
        let Some((addr, shift)) = Self::pull_addr_shift() else {
            return;
        };
        let mask = 0b11 << shift;
        let value = (analog::read(addr) & !mask) | (pull.bits() << shift);
        analog::write(addr, value);
    }

    pub fn set_interrupt_edge(&mut self, edge: InterruptEdge) {
        Self::set_interrupt_polarity(edge);
    }

    pub fn enable_interrupt(&mut self, route: InterruptRoute, edge: InterruptEdge) {
        Self::set_interrupt_polarity(edge);
        match route {
            InterruptRoute::Core => {
                Self::set_wakeup_irq_flag(1 << 3, true);
                Self::clear_irq_src(1 << 18);
                Self::write_irq_mask(1 << 18);
            }
            InterruptRoute::Risc0 => {
                Self::clear_irq_src(1 << 21);
                Self::write_irq_mask(1 << 21);
            }
            InterruptRoute::Risc1 => {
                Self::clear_irq_src(1 << 22);
                Self::write_irq_mask(1 << 22);
            }
        }
        Self::modify_raw_reg(Self::route_reg(route), Self::mask(), true);
    }

    pub fn disable_interrupt(&mut self, route: InterruptRoute) {
        Self::modify_raw_reg(Self::route_reg(route), Self::mask(), false);
    }

    pub fn enable_wakeup(&mut self, edge: InterruptEdge) {
        Self::set_interrupt_polarity(edge);
        Self::set_wakeup_irq_flag(1 << 2, true);
        Self::modify_raw_reg(Self::route_reg(InterruptRoute::Core), Self::mask(), true);
    }

    pub fn disable_wakeup(&mut self) {
        Self::modify_raw_reg(Self::route_reg(InterruptRoute::Core), Self::mask(), false);
    }
}

impl<const PORT: u8, const BIT: u8> Pin<PORT, BIT, Output> {
    pub fn set_high(&mut self) {
        Self::write_data(true);
    }

    pub fn set_low(&mut self) {
        Self::write_data(false);
    }

    pub fn toggle(&mut self) {
        Self::write_data(!Self::read_output_level());
    }

    pub fn is_set_high(&self) -> bool {
        Self::read_output_level()
    }

    pub fn is_set_low(&self) -> bool {
        !Self::read_output_level()
    }
}

impl<const PORT: u8, const BIT: u8> Pin<PORT, BIT, Input> {
    pub fn is_high(&self) -> bool {
        Self::read_input_level()
    }

    pub fn is_low(&self) -> bool {
        !Self::read_input_level()
    }
}

impl<const PORT: u8, const BIT: u8, MODE> ErrorType for Pin<PORT, BIT, MODE> {
    type Error = Infallible;
}

impl<const PORT: u8, const BIT: u8> EhOutputPin for Pin<PORT, BIT, Output> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Pin::<PORT, BIT, Output>::set_low(self);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Pin::<PORT, BIT, Output>::set_high(self);
        Ok(())
    }
}

impl<const PORT: u8, const BIT: u8> EhStatefulOutputPin for Pin<PORT, BIT, Output> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::<PORT, BIT, Output>::is_set_high(self))
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::<PORT, BIT, Output>::is_set_low(self))
    }
}

impl<const PORT: u8, const BIT: u8> EhInputPin for Pin<PORT, BIT, Input> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::<PORT, BIT, Input>::is_high(self))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(Pin::<PORT, BIT, Input>::is_low(self))
    }
}
