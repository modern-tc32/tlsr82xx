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
const MUX_BASE: usize = 0x0080_05a8;

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
#[repr(u8)]
pub enum PinFunction {
    Gpio = 0,
    Mspi = 1,
    Swire = 2,
    Uart = 3,
    I2c = 4,
    Spi = 5,
    I2s = 6,
    Dmic = 8,
    Sdm = 9,
    Usb = 10,
    Cmp = 12,
    Ats = 13,
    Pwm0 = 20,
    Pwm1 = 21,
    Pwm2 = 22,
    Pwm3 = 23,
    Pwm4 = 24,
    Pwm5 = 25,
    Pwm0N = 26,
    Pwm1N = 27,
    Pwm2N = 28,
    Pwm3N = 29,
    Pwm4N = 30,
    Pwm5N = 31,
    TxCyc2Pa = 32,
    RxCyc2Lna = 33,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct RawPin(u16);

impl RawPin {
    #[inline(always)]
    pub const fn from_parts(port: u8, bit: u8) -> Self {
        Self(((port as u16) << 8) | (1u16 << bit))
    }

    #[inline(always)]
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    pub fn try_from_u16(raw: u16) -> Result<Self, PinmuxError> {
        if decode_raw_pin_checked_u16(raw).is_some() {
            Ok(Self(raw))
        } else {
            Err(PinmuxError::InvalidRawPin(raw))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PinmuxError {
    InvalidRawPin(u16),
    UnsupportedPair {
        raw_pin: RawPin,
        function: PinFunction,
    },
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
    pub const fn raw_pin() -> RawPin {
        RawPin::from_parts(PORT, BIT)
    }

    pub fn set_function(&mut self, function: PinFunction) -> Result<(), PinmuxError> {
        set_function_raw(Self::raw_pin(), function)
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
                let addr = 0x0e + (PORT * 2) + (BIT / 4);
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
        #[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
        {
            match PORT {
                PORT_B => {
                    let mut value = analog::read(0xbd);
                    if enabled {
                        value |= Self::mask();
                    } else {
                        value &= !Self::mask();
                    }
                    analog::write(0xbd, value);
                }
                PORT_C => {
                    let mut value = analog::read(0xc0);
                    if enabled {
                        value |= Self::mask();
                    } else {
                        value &= !Self::mask();
                    }
                    analog::write(0xc0, value);
                }
                _ => Self::modify_reg(0x01, enabled),
            }
            return;
        }

        #[allow(unreachable_code)]
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
        #[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
        {
            match PORT {
                PORT_B => {
                    let mut value = analog::read(0xbf);
                    if strong {
                        value |= Self::mask();
                    } else {
                        value &= !Self::mask();
                    }
                    analog::write(0xbf, value);
                }
                PORT_C => {
                    let mut value = analog::read(0xc2);
                    if strong {
                        value |= Self::mask();
                    } else {
                        value &= !Self::mask();
                    }
                    analog::write(0xc2, value);
                }
                _ => Self::modify_reg(0x05, strong),
            }
            return;
        }

        #[allow(unreachable_code)]
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

#[inline(always)]
fn decode_raw_pin(raw_pin: RawPin) -> (u8, u8, u8) {
    let raw = raw_pin.as_u16();
    let port = (raw >> 8) as u8;
    let mask = raw as u8;
    debug_assert!(mask.is_power_of_two());
    let bit = mask.trailing_zeros() as u8;
    (port, bit, mask)
}

#[inline(always)]
fn decode_raw_pin_checked_u16(raw_pin: u16) -> Option<(u8, u8, u8)> {
    let port = (raw_pin >> 8) as u8;
    let mask = raw_pin as u8;
    if !mask.is_power_of_two() {
        return None;
    }
    let bit = mask.trailing_zeros() as u8;
    if port > PORT_E {
        return None;
    }
    if port == PORT_E && bit > 3 {
        return None;
    }
    Some((port, bit, mask))
}

#[inline(always)]
fn decode_raw_pin_checked(raw_pin: RawPin) -> Option<(u8, u8, u8)> {
    decode_raw_pin_checked_u16(raw_pin.as_u16())
}

#[inline(always)]
fn reg_raw(port: u8, offset: usize) -> *mut u8 {
    (GPIO_BASE + ((port as usize) << 3) + offset) as *mut u8
}

#[inline(always)]
fn modify_raw_port_reg(port: u8, offset: usize, mask: u8, set: bool) {
    unsafe {
        let reg = reg_raw(port, offset);
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
fn write_mux_selector(port: u8, bit: u8, selector: u8) {
    let reg = (MUX_BASE + (port as usize * 2) + (bit as usize / 4)) as *mut u8;
    let shift = (bit % 4) * 2;
    let field_mask = !(0b11 << shift);
    unsafe {
        let value = (core::ptr::read_volatile(reg.cast_const()) & field_mask)
            | ((selector & 0b11) << shift);
        core::ptr::write_volatile(reg, value);
    }
}

enum PinmuxMode {
    Selector(u8),
    FixedNoMux,
}

#[inline(always)]
fn pinmux_mode_for(raw_pin: RawPin, function: PinFunction) -> Option<PinmuxMode> {
    match (raw_pin.as_u16(), function) {
        (_, PinFunction::Gpio) => None,
        (0x0001, PinFunction::Dmic) => Some(PinmuxMode::Selector(0b00)), // PA0
        (0x0001, PinFunction::Pwm0N) => Some(PinmuxMode::Selector(0b01)), // PA0
        (0x0001, PinFunction::Uart) => Some(PinmuxMode::Selector(0b10)), // PA0
        (0x0002, PinFunction::Dmic) => Some(PinmuxMode::Selector(0b00)), // PA1
        (0x0002, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PA1
        (0x0002, PinFunction::I2s) => Some(PinmuxMode::Selector(0b10)),  // PA1
        (0x0004, PinFunction::Spi) => Some(PinmuxMode::Selector(0b00)),  // PA2
        (0x0004, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PA2
        (0x0004, PinFunction::Pwm0) => Some(PinmuxMode::Selector(0b10)), // PA2
        (0x0008, PinFunction::Spi) => Some(PinmuxMode::Selector(0b00)),  // PA3
        (0x0008, PinFunction::I2c) => Some(PinmuxMode::Selector(0b00)),  // PA3
        (0x0008, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PA3
        (0x0008, PinFunction::Pwm1) => Some(PinmuxMode::Selector(0b10)), // PA3
        (0x0010, PinFunction::Spi) => Some(PinmuxMode::Selector(0b00)),  // PA4
        (0x0010, PinFunction::I2c) => Some(PinmuxMode::Selector(0b00)),  // PA4
        (0x0010, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PA4
        (0x0010, PinFunction::Pwm2) => Some(PinmuxMode::Selector(0b10)), // PA4
        (0x0020, PinFunction::Usb) => Some(PinmuxMode::FixedNoMux),      // PA5
        (0x0040, PinFunction::Usb) => Some(PinmuxMode::FixedNoMux),      // PA6
        (0x0080, PinFunction::Swire) => Some(PinmuxMode::Selector(0b00)), // PA7
        (0x0080, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PA7
        (0x0101, PinFunction::Pwm3) => Some(PinmuxMode::Selector(0b00)), // PB0
        (0x0101, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PB0
        (0x0101, PinFunction::Ats) => Some(PinmuxMode::Selector(0b10)),  // PB0
        (0x0102, PinFunction::Pwm4) => Some(PinmuxMode::Selector(0b00)), // PB1
        (0x0102, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PB1
        (0x0102, PinFunction::Ats) => Some(PinmuxMode::Selector(0b10)),  // PB1
        (0x0104, PinFunction::Pwm5) => Some(PinmuxMode::Selector(0b00)), // PB2
        (0x0104, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PB2
        (0x0104, PinFunction::RxCyc2Lna) => Some(PinmuxMode::Selector(0b10)), // PB2
        (0x0108, PinFunction::Pwm0N) => Some(PinmuxMode::Selector(0b00)), // PB3
        (0x0108, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PB3
        (0x0108, PinFunction::TxCyc2Pa) => Some(PinmuxMode::Selector(0b10)), // PB3
        (0x0110, PinFunction::Sdm) => Some(PinmuxMode::Selector(0b00)),  // PB4
        (0x0110, PinFunction::Pwm4) => Some(PinmuxMode::Selector(0b01)), // PB4
        (0x0110, PinFunction::Cmp) => Some(PinmuxMode::Selector(0b10)),  // PB4
        (0x0120, PinFunction::Sdm) => Some(PinmuxMode::Selector(0b00)),  // PB5
        (0x0120, PinFunction::Pwm5) => Some(PinmuxMode::Selector(0b01)), // PB5
        (0x0120, PinFunction::Cmp) => Some(PinmuxMode::Selector(0b10)),  // PB5
        (0x0140, PinFunction::Sdm) => Some(PinmuxMode::Selector(0b00)),  // PB6
        (0x0140, PinFunction::Spi) => Some(PinmuxMode::Selector(0b01)),  // PB6
        (0x0140, PinFunction::I2c) => Some(PinmuxMode::Selector(0b01)),  // PB6
        (0x0140, PinFunction::Uart) => Some(PinmuxMode::Selector(0b10)), // PB6
        (0x0180, PinFunction::Sdm) => Some(PinmuxMode::Selector(0b00)),  // PB7
        (0x0180, PinFunction::Spi) => Some(PinmuxMode::Selector(0b01)),  // PB7
        (0x0180, PinFunction::Uart) => Some(PinmuxMode::Selector(0b10)), // PB7
        (0x0201, PinFunction::I2c) => Some(PinmuxMode::Selector(0b00)),  // PC0
        (0x0201, PinFunction::Pwm4N) => Some(PinmuxMode::Selector(0b01)), // PC0
        (0x0201, PinFunction::Uart) => Some(PinmuxMode::Selector(0b10)), // PC0
        (0x0202, PinFunction::I2c) => Some(PinmuxMode::Selector(0b00)),  // PC1
        (0x0202, PinFunction::Pwm1N) => Some(PinmuxMode::Selector(0b01)), // PC1
        (0x0202, PinFunction::Pwm0) => Some(PinmuxMode::Selector(0b10)), // PC1
        (0x0204, PinFunction::Pwm0) => Some(PinmuxMode::Selector(0b00)), // PC2
        (0x0204, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PC2
        (0x0204, PinFunction::I2c) => Some(PinmuxMode::Selector(0b10)),  // PC2
        (0x0208, PinFunction::Pwm1) => Some(PinmuxMode::Selector(0b00)), // PC3
        (0x0208, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PC3
        (0x0208, PinFunction::I2c) => Some(PinmuxMode::Selector(0b10)),  // PC3
        (0x0210, PinFunction::Pwm2) => Some(PinmuxMode::Selector(0b00)), // PC4
        (0x0210, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PC4
        (0x0210, PinFunction::Pwm0N) => Some(PinmuxMode::Selector(0b10)), // PC4
        (0x0220, PinFunction::Pwm3N) => Some(PinmuxMode::Selector(0b00)), // PC5
        (0x0220, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PC5
        (0x0220, PinFunction::Ats) => Some(PinmuxMode::Selector(0b10)),  // PC5
        (0x0240, PinFunction::RxCyc2Lna) => Some(PinmuxMode::Selector(0b00)), // PC6
        (0x0240, PinFunction::Ats) => Some(PinmuxMode::Selector(0b01)),  // PC6
        (0x0240, PinFunction::Pwm4N) => Some(PinmuxMode::Selector(0b10)), // PC6
        (0x0280, PinFunction::TxCyc2Pa) => Some(PinmuxMode::Selector(0b00)), // PC7
        (0x0280, PinFunction::Ats) => Some(PinmuxMode::Selector(0b01)),  // PC7
        (0x0280, PinFunction::Pwm5N) => Some(PinmuxMode::Selector(0b10)), // PC7
        (0x0301, PinFunction::RxCyc2Lna) => Some(PinmuxMode::Selector(0b00)), // PD0
        (0x0301, PinFunction::Cmp) => Some(PinmuxMode::Selector(0b01)),  // PD0
        (0x0301, PinFunction::Uart) => Some(PinmuxMode::Selector(0b10)), // PD0
        (0x0302, PinFunction::TxCyc2Pa) => Some(PinmuxMode::Selector(0b00)), // PD1
        (0x0302, PinFunction::Cmp) => Some(PinmuxMode::Selector(0b01)),  // PD1
        (0x0302, PinFunction::Uart) => Some(PinmuxMode::Selector(0b10)), // PD1
        (0x0304, PinFunction::Spi) => Some(PinmuxMode::Selector(0b00)),  // PD2
        (0x0304, PinFunction::I2s) => Some(PinmuxMode::Selector(0b01)),  // PD2
        (0x0304, PinFunction::Pwm3) => Some(PinmuxMode::Selector(0b10)), // PD2
        (0x0308, PinFunction::Pwm1N) => Some(PinmuxMode::Selector(0b00)), // PD3
        (0x0308, PinFunction::I2s) => Some(PinmuxMode::Selector(0b01)),  // PD3
        (0x0308, PinFunction::Uart) => Some(PinmuxMode::Selector(0b10)), // PD3
        (0x0310, PinFunction::Swire) => Some(PinmuxMode::Selector(0b00)), // PD4
        (0x0310, PinFunction::I2s) => Some(PinmuxMode::Selector(0b01)),  // PD4
        (0x0310, PinFunction::Pwm2N) => Some(PinmuxMode::Selector(0b10)), // PD4
        (0x0320, PinFunction::Pwm0) => Some(PinmuxMode::Selector(0b00)), // PD5
        (0x0320, PinFunction::Cmp) => Some(PinmuxMode::Selector(0b01)),  // PD5
        (0x0320, PinFunction::Pwm0N) => Some(PinmuxMode::Selector(0b10)), // PD5
        (0x0340, PinFunction::Spi) => Some(PinmuxMode::Selector(0b00)),  // PD6
        (0x0340, PinFunction::Uart) => Some(PinmuxMode::Selector(0b01)), // PD6
        (0x0340, PinFunction::Ats) => Some(PinmuxMode::Selector(0b10)),  // PD6
        (0x0380, PinFunction::Spi) => Some(PinmuxMode::Selector(0b00)),  // PD7
        (0x0380, PinFunction::I2c) => Some(PinmuxMode::Selector(0b00)),  // PD7
        (0x0380, PinFunction::I2s) => Some(PinmuxMode::Selector(0b01)),  // PD7
        (0x0380, PinFunction::Uart) => Some(PinmuxMode::Selector(0b10)), // PD7
        (0x0401, PinFunction::Mspi) => Some(PinmuxMode::FixedNoMux),     // PE0
        (0x0402, PinFunction::Mspi) => Some(PinmuxMode::FixedNoMux),     // PE1
        (0x0404, PinFunction::Mspi) => Some(PinmuxMode::FixedNoMux),     // PE2
        (0x0408, PinFunction::Mspi) => Some(PinmuxMode::FixedNoMux),     // PE3
        _ => None,
    }
}

pub(crate) fn set_function_raw(raw_pin: RawPin, function: PinFunction) -> Result<(), PinmuxError> {
    let Some((port, bit, mask)) = decode_raw_pin_checked(raw_pin) else {
        return Err(PinmuxError::InvalidRawPin(raw_pin.as_u16()));
    };
    if matches!(function, PinFunction::Gpio) {
        modify_raw_port_reg(port, 0x06, mask, true);
        return Ok(());
    }

    let Some(mode) = pinmux_mode_for(raw_pin, function) else {
        return Err(PinmuxError::UnsupportedPair { raw_pin, function });
    };
    match mode {
        PinmuxMode::Selector(selector) => write_mux_selector(port, bit, selector),
        PinmuxMode::FixedNoMux => {
            if matches!(function, PinFunction::Usb) {
                set_input_enabled_raw(raw_pin, true);
            }
        }
    }
    modify_raw_port_reg(port, 0x06, mask, false);
    Ok(())
}

pub fn set_function_for_raw_pin(raw_pin: RawPin, function: PinFunction) -> Result<(), PinmuxError> {
    set_function_raw(raw_pin, function)
}

pub(crate) fn set_input_enabled_raw(raw_pin: RawPin, enabled: bool) {
    let (port, _, mask) = decode_raw_pin(raw_pin);
    #[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
    {
        match port {
            PORT_B => {
                let mut reg = analog::read(0xbd);
                if enabled {
                    reg |= mask;
                } else {
                    reg &= !mask;
                }
                analog::write(0xbd, reg);
                return;
            }
            PORT_C => {
                let mut reg = analog::read(0xc0);
                if enabled {
                    reg |= mask;
                } else {
                    reg &= !mask;
                }
                analog::write(0xc0, reg);
                return;
            }
            _ => {}
        }
    }

    modify_raw_port_reg(port, 0x01, mask, enabled);
}

pub(crate) fn set_output_enabled_raw(raw_pin: RawPin, enabled: bool) {
    let (port, _, mask) = decode_raw_pin(raw_pin);
    modify_raw_port_reg(port, 0x02, mask, !enabled);
}

pub(crate) fn write_data_raw(raw_pin: RawPin, high: bool) {
    let (port, _, mask) = decode_raw_pin(raw_pin);
    modify_raw_port_reg(port, 0x03, mask, high);
}

pub(crate) fn set_pull_resistor_raw(raw_pin: RawPin, pull: analog::Pull) {
    let (port, bit, _) = decode_raw_pin(raw_pin);
    let Some((addr, shift)) = pull_addr_shift_raw(port, bit) else {
        return;
    };
    let mask = 0b11 << shift;
    let value = (analog::read(addr) & !mask) | (pull.bits() << shift);
    analog::write(addr, value);
}

#[inline(always)]
fn pull_addr_shift_raw(port: u8, bit: u8) -> Option<(u8, u8)> {
    #[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
    {
        if port <= PORT_D {
            let addr = 0x0e + (port * 2) + (bit / 4);
            let shift = (bit % 4) * 2;
            return Some((addr, shift));
        }
        return None;
    }

    #[cfg(feature = "chip-826x")]
    {
        return match (port, bit) {
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

impl<const PORT: u8, const BIT: u8> Pin<PORT, BIT, Output> {
    #[inline(always)]
    fn set_output_high(&mut self) {
        Self::write_data(true);
    }

    #[inline(always)]
    fn set_output_low(&mut self) {
        Self::write_data(false);
    }

    #[inline(always)]
    fn toggle_output(&mut self) {
        Self::write_data(!Self::read_output_level());
    }

    #[inline(always)]
    fn output_is_set_high(&self) -> bool {
        Self::read_output_level()
    }

    #[inline(always)]
    fn output_is_set_low(&self) -> bool {
        !Self::read_output_level()
    }
}

impl<const PORT: u8, const BIT: u8> Pin<PORT, BIT, Input> {
    #[inline(always)]
    fn input_is_high(&self) -> bool {
        Self::read_input_level()
    }

    #[inline(always)]
    fn input_is_low(&self) -> bool {
        !Self::read_input_level()
    }
}

impl<const PORT: u8, const BIT: u8, MODE> ErrorType for Pin<PORT, BIT, MODE> {
    type Error = Infallible;
}

impl<const PORT: u8, const BIT: u8> EhOutputPin for Pin<PORT, BIT, Output> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_output_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_output_high();
        Ok(())
    }
}

impl<const PORT: u8, const BIT: u8> EhStatefulOutputPin for Pin<PORT, BIT, Output> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.output_is_set_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.output_is_set_low())
    }

    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle_output();
        Ok(())
    }
}

impl<const PORT: u8, const BIT: u8> EhInputPin for Pin<PORT, BIT, Input> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.input_is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.input_is_low())
    }
}

#[cfg(test)]
mod tests {
    use super::{pinmux_mode_for, PinFunction, PinmuxMode, RawPin};

    #[test]
    fn pb1_uart_selector_is_slot_01() {
        assert!(matches!(
            pinmux_mode_for(RawPin::try_from_u16(0x0102).unwrap(), PinFunction::Uart),
            Some(PinmuxMode::Selector(0b01))
        ));
    }

    #[test]
    fn rf_alt_default_slot_cases_match_vendor() {
        assert!(matches!(
            pinmux_mode_for(RawPin::try_from_u16(0x0280).unwrap(), PinFunction::TxCyc2Pa),
            Some(PinmuxMode::Selector(0b00))
        ));
        assert!(matches!(
            pinmux_mode_for(RawPin::try_from_u16(0x0301).unwrap(), PinFunction::RxCyc2Lna),
            Some(PinmuxMode::Selector(0b00))
        ));
    }

    #[test]
    fn pa5_usb_is_fixed_no_mux() {
        assert!(matches!(
            pinmux_mode_for(RawPin::try_from_u16(0x0020).unwrap(), PinFunction::Usb),
            Some(PinmuxMode::FixedNoMux)
        ));
    }

    #[test]
    fn unsupported_pair_is_rejected() {
        assert!(pinmux_mode_for(RawPin::try_from_u16(0x0104).unwrap(), PinFunction::Pwm0).is_none());
    }

    #[test]
    fn raw_pin_try_from_u16_validates_input() {
        assert!(RawPin::try_from_u16(0x0104).is_ok());
        assert!(RawPin::try_from_u16(0x0000).is_err());
        assert!(RawPin::try_from_u16(0x0410).is_err());
    }
}
