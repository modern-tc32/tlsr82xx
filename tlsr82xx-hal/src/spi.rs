use embedded_hal::spi::{
    Error as EhError, ErrorKind, ErrorType, Mode, Phase, Polarity, SpiBus,
};

use crate::clock;
use crate::gpio;
use crate::mmio::reg8;
use crate::regs8258::{
    FLD_CLK0_SPI_EN, FLD_SPI_BUSY, FLD_SPI_DATA_OUT_DIS, FLD_SPI_ENABLE,
    FLD_SPI_MASTER_MODE_EN, FLD_SPI_RD, FLD_SPI_SHARE_MODE, REG_CLK_EN0, REG_SPI_CTRL,
    REG_SPI_DATA, REG_SPI_INV_CLK, REG_SPI_SP,
};

const SPI_TIMEOUT_CYCLES: usize = 100_000;
const SPI_CLOCK_DIVIDER_MAX: u8 = 0x7f;
const SPI_MODE0_INV_CLK: u8 = 0;
const SPI_MODE1_INV_CLK: u8 = 1;
const SPI_MODE2_INV_CLK: u8 = 2;
const SPI_MODE3_INV_CLK: u8 = 3;
const SPI_TX_DUMMY_BYTE: u8 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SpiPinGroup {
    A2A3A4D6 = 0,
    B6B7D2D7 = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    pub frequency_hz: u32,
    pub mode: Mode,
    pub pin_group: SpiPinGroup,
    pub share_mode: bool,
}

impl Config {
    pub const fn new(pin_group: SpiPinGroup, frequency_hz: u32, mode: Mode) -> Self {
        Self {
            frequency_hz,
            mode,
            pin_group,
            share_mode: false,
        }
    }

    pub const fn with_share_mode(mut self, share_mode: bool) -> Self {
        self.share_mode = share_mode;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpiError {
    Timeout,
    Other,
}

impl EhError for SpiError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Timeout | Self::Other => ErrorKind::Other,
        }
    }
}

pub struct Spi {
    frequency_hz: u32,
    mode: Mode,
}

pub trait SpiPins {
    const PIN_GROUP: SpiPinGroup;
}

impl<'a, A2Mode, A3Mode, A4Mode, D6Mode> SpiPins
    for (
        &'a mut gpio::PA2<A2Mode>,
        &'a mut gpio::PA3<A3Mode>,
        &'a mut gpio::PA4<A4Mode>,
        &'a mut gpio::PD6<D6Mode>,
    )
{
    const PIN_GROUP: SpiPinGroup = SpiPinGroup::A2A3A4D6;
}

impl<'a, B6Mode, B7Mode, D2Mode, D7Mode> SpiPins
    for (
        &'a mut gpio::PB6<B6Mode>,
        &'a mut gpio::PB7<B7Mode>,
        &'a mut gpio::PD2<D2Mode>,
        &'a mut gpio::PD7<D7Mode>,
    )
{
    const PIN_GROUP: SpiPinGroup = SpiPinGroup::B6B7D2D7;
}

impl Spi {
    pub fn new(config: Config) -> Self {
        configure_gpio(config.pin_group);
        let mut spi = Self {
            frequency_hz: config.frequency_hz,
            mode: config.mode,
        };
        spi.init_registers(config.share_mode);
        spi
    }

    pub fn with_pins<PINS: SpiPins>(pins: PINS, frequency_hz: u32, mode: Mode) -> Self {
        let _ = pins;
        Self::new(Config::new(PINS::PIN_GROUP, frequency_hz, mode))
    }

    pub fn frequency_hz(&self) -> u32 {
        self.frequency_hz
    }

    pub fn set_frequency(&mut self, frequency_hz: u32) {
        self.frequency_hz = frequency_hz.max(1);
        self.write_clock_divider();
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.write_mode();
    }

    pub fn set_share_mode(&mut self, enabled: bool) {
        unsafe {
            let reg = reg8(REG_SPI_CTRL);
            let mut bits = core::ptr::read_volatile(reg.cast_const());
            if enabled {
                bits |= FLD_SPI_SHARE_MODE;
            } else {
                bits &= !FLD_SPI_SHARE_MODE;
            }
            core::ptr::write_volatile(reg, bits);
        }
    }

    fn init_registers(&mut self, share_mode: bool) {
        unsafe {
            let clk_en0 = reg8(REG_CLK_EN0);
            core::ptr::write_volatile(
                clk_en0,
                core::ptr::read_volatile(clk_en0.cast_const()) | FLD_CLK0_SPI_EN,
            );
        }
        self.write_clock_divider();
        unsafe {
            core::ptr::write_volatile(reg8(REG_SPI_SP), FLD_SPI_ENABLE | compute_divider(self.frequency_hz));
            let ctrl = reg8(REG_SPI_CTRL);
            let mut bits = core::ptr::read_volatile(ctrl.cast_const()) | FLD_SPI_MASTER_MODE_EN;
            if share_mode {
                bits |= FLD_SPI_SHARE_MODE;
            } else {
                bits &= !FLD_SPI_SHARE_MODE;
            }
            core::ptr::write_volatile(ctrl, bits);
        }
        self.write_mode();
    }

    fn write_clock_divider(&self) {
        unsafe {
            core::ptr::write_volatile(
                reg8(REG_SPI_SP),
                FLD_SPI_ENABLE | compute_divider(self.frequency_hz),
            );
        }
    }

    fn write_mode(&self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_SPI_INV_CLK), encode_mode(self.mode));
        }
    }

    fn wait_not_busy(&self) -> Result<(), SpiError> {
        for _ in 0..SPI_TIMEOUT_CYCLES {
            let bits = unsafe { core::ptr::read_volatile(reg8(REG_SPI_CTRL).cast_const()) };
            if (bits & FLD_SPI_BUSY) == 0 {
                return Ok(());
            }
        }
        Err(SpiError::Timeout)
    }

    fn write_setup(&mut self) {
        unsafe {
            let reg = reg8(REG_SPI_CTRL);
            let mut bits = core::ptr::read_volatile(reg.cast_const());
            bits &= !FLD_SPI_DATA_OUT_DIS;
            bits &= !FLD_SPI_RD;
            core::ptr::write_volatile(reg, bits);
        }
    }

    fn read_setup(&mut self) {
        unsafe {
            let reg = reg8(REG_SPI_CTRL);
            let mut bits = core::ptr::read_volatile(reg.cast_const());
            bits |= FLD_SPI_DATA_OUT_DIS;
            bits |= FLD_SPI_RD;
            core::ptr::write_volatile(reg, bits);
        }
    }

    fn write_byte(&mut self, byte: u8) -> Result<u8, SpiError> {
        unsafe {
            core::ptr::write_volatile(reg8(REG_SPI_DATA), byte);
        }
        self.wait_not_busy()?;
        Ok(unsafe { core::ptr::read_volatile(reg8(REG_SPI_DATA).cast_const()) })
    }

    fn prime_read(&mut self) -> Result<(), SpiError> {
        let _ = unsafe { core::ptr::read_volatile(reg8(REG_SPI_DATA).cast_const()) };
        self.wait_not_busy()
    }
}

impl ErrorType for Spi {
    type Error = SpiError;
}

impl SpiBus<u8> for Spi {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.read_setup();
        if words.is_empty() {
            return Ok(());
        }
        self.prime_read()?;
        for word in words.iter_mut() {
            *word = unsafe { core::ptr::read_volatile(reg8(REG_SPI_DATA).cast_const()) };
            self.wait_not_busy()?;
        }
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write_setup();
        for &word in words {
            let _ = self.write_byte(word)?;
        }
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.write_setup();
        let total = read.len().max(write.len());
        for index in 0..total {
            let tx = write.get(index).copied().unwrap_or(SPI_TX_DUMMY_BYTE);
            let rx = self.write_byte(tx)?;
            if let Some(slot) = read.get_mut(index) {
                *slot = rx;
            }
        }
        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.write_setup();
        for word in words.iter_mut() {
            *word = self.write_byte(*word)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.wait_not_busy()
    }
}

fn compute_divider(frequency_hz: u32) -> u8 {
    let sys_hz = u32::from(clock::current_mhz()) * 1_000_000;
    let divider = (sys_hz / (frequency_hz.max(1) * 2)).saturating_sub(1);
    divider.min(u32::from(SPI_CLOCK_DIVIDER_MAX)) as u8
}

fn encode_mode(mode: Mode) -> u8 {
    match (mode.polarity, mode.phase) {
        (Polarity::IdleLow, Phase::CaptureOnFirstTransition) => SPI_MODE0_INV_CLK,
        (Polarity::IdleHigh, Phase::CaptureOnFirstTransition) => SPI_MODE1_INV_CLK,
        (Polarity::IdleLow, Phase::CaptureOnSecondTransition) => SPI_MODE2_INV_CLK,
        (Polarity::IdleHigh, Phase::CaptureOnSecondTransition) => SPI_MODE3_INV_CLK,
    }
}

fn configure_gpio(pin_group: SpiPinGroup) {
    unsafe extern "C" {
        fn spi_master_gpio_set(pin_group: u32);
    }

    unsafe {
        spi_master_gpio_set(pin_group as u32);
    }
}
