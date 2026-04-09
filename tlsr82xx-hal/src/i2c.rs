use embedded_hal::i2c::{
    Error as EhError, ErrorKind, ErrorType, I2c as EhI2c, NoAcknowledgeSource, Operation,
    SevenBitAddress,
};

use crate::clock;
use crate::gpio;
use crate::mmio::reg8;
use crate::regs8258::{
    FLD_CLK0_I2C_EN, FLD_I2C_CMD_ACK, FLD_I2C_CMD_BUSY, FLD_I2C_CMD_DI, FLD_I2C_CMD_DO,
    FLD_I2C_CMD_ID, FLD_I2C_CMD_READ_ID, FLD_I2C_CMD_START, FLD_I2C_CMD_STOP, FLD_I2C_HOLD_MASTER,
    FLD_I2C_MASTER_EN, FLD_I2C_NAK, FLD_I2C_WRITE_READ_BIT, FLD_RST0_I2C, FLD_SPI_ENABLE,
    REG_CLK_EN0, REG_I2C_CTRL, REG_I2C_DI, REG_I2C_DO, REG_I2C_ID, REG_I2C_MODE, REG_I2C_SPEED,
    REG_I2C_STATUS, REG_RST0, REG_SPI_SP,
};

const I2C_MAX_DIVIDER: u8 = u8::MAX;
const I2C_DEFAULT_FREQ_HZ: u32 = 100_000;
const I2C_MAX_FREQ_HZ: u32 = 400_000;
const I2C_TIMEOUT_CYCLES: usize = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum I2cPinGroup {
    A3A4 = 0,
    B6D7 = 1,
    C0C1 = 2,
    C2C3 = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    pub frequency_hz: u32,
    pub pin_group: I2cPinGroup,
}

impl Config {
    pub const fn new(pin_group: I2cPinGroup, frequency_hz: u32) -> Self {
        Self {
            pin_group,
            frequency_hz,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(I2cPinGroup::C0C1, I2C_DEFAULT_FREQ_HZ)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum I2cError {
    Timeout,
    NoAcknowledge(NoAcknowledgeSource),
    BusBusy,
    Other,
}

impl EhError for I2cError {
    fn kind(&self) -> ErrorKind {
        match *self {
            Self::Timeout => ErrorKind::Other,
            Self::NoAcknowledge(source) => ErrorKind::NoAcknowledge(source),
            Self::BusBusy => ErrorKind::Bus,
            Self::Other => ErrorKind::Other,
        }
    }
}

pub struct I2c {
    frequency_hz: u32,
}

pub trait I2cPins {
    const PIN_GROUP: I2cPinGroup;
}

impl<'a, SdaMode, SclMode> I2cPins for (&'a mut gpio::PA3<SdaMode>, &'a mut gpio::PA4<SclMode>) {
    const PIN_GROUP: I2cPinGroup = I2cPinGroup::A3A4;
}
impl<'a, SdaMode, SclMode> I2cPins for (&'a mut gpio::PB6<SdaMode>, &'a mut gpio::PD7<SclMode>) {
    const PIN_GROUP: I2cPinGroup = I2cPinGroup::B6D7;
}
impl<'a, SdaMode, SclMode> I2cPins for (&'a mut gpio::PC0<SdaMode>, &'a mut gpio::PC1<SclMode>) {
    const PIN_GROUP: I2cPinGroup = I2cPinGroup::C0C1;
}
impl<'a, SdaMode, SclMode> I2cPins for (&'a mut gpio::PC2<SdaMode>, &'a mut gpio::PC3<SclMode>) {
    const PIN_GROUP: I2cPinGroup = I2cPinGroup::C2C3;
}

impl I2c {
    pub fn new(config: Config) -> Self {
        configure_gpio(config.pin_group);
        let mut i2c = Self {
            frequency_hz: clamp_frequency(config.frequency_hz),
        };
        i2c.init_registers();
        i2c
    }

    pub fn with_frequency(pin_group: I2cPinGroup, frequency_hz: u32) -> Self {
        Self::new(Config::new(pin_group, frequency_hz))
    }

    pub fn with_pins<PINS: I2cPins>(pins: PINS, frequency_hz: u32) -> Self {
        let _ = pins;
        Self::new(Config::new(PINS::PIN_GROUP, frequency_hz))
    }

    pub fn frequency_hz(&self) -> u32 {
        self.frequency_hz
    }

    pub fn set_frequency(&mut self, frequency_hz: u32) {
        self.frequency_hz = clamp_frequency(frequency_hz);
        self.write_speed_divider();
    }

    fn init_registers(&mut self) {
        enable_clock();
        disable_spi_pad_mode();
        self.write_speed_divider();

        unsafe {
            let mode = core::ptr::read_volatile(reg8(REG_I2C_MODE).cast_const());
            core::ptr::write_volatile(
                reg8(REG_I2C_MODE),
                (mode | FLD_I2C_MASTER_EN) & !FLD_I2C_HOLD_MASTER,
            );
        }
    }

    fn write_speed_divider(&self) {
        let sys_hz = u32::from(clock::current_mhz()) * 1_000_000;
        let divider = ((sys_hz / (4 * self.frequency_hz)).max(1)).min(u32::from(I2C_MAX_DIVIDER));
        unsafe {
            core::ptr::write_volatile(reg8(REG_I2C_SPEED), divider as u8);
        }
    }

    fn wait_while_busy(&self) -> Result<u8, I2cError> {
        for _ in 0..I2C_TIMEOUT_CYCLES {
            let status = unsafe { core::ptr::read_volatile(reg8(REG_I2C_STATUS).cast_const()) };
            if (status & FLD_I2C_CMD_BUSY) == 0 {
                return Ok(status);
            }
        }
        Err(I2cError::Timeout)
    }

    fn stop(&self) -> Result<(), I2cError> {
        unsafe {
            core::ptr::write_volatile(reg8(REG_I2C_CTRL), FLD_I2C_CMD_STOP);
        }
        let _ = self.wait_while_busy()?;
        Ok(())
    }

    fn reset_bus(&self) {
        unsafe {
            core::ptr::write_volatile(reg8(REG_RST0), FLD_RST0_I2C);
            core::ptr::write_volatile(reg8(REG_RST0), 0);
        }
    }

    fn start_write(&self, address: SevenBitAddress) -> Result<(), I2cError> {
        self.start_id(address << 1, false)
    }

    fn start_read(&self, address: SevenBitAddress) -> Result<(), I2cError> {
        self.start_id((address << 1) | FLD_I2C_WRITE_READ_BIT, true)
    }

    fn start_id(&self, address_byte: u8, is_read: bool) -> Result<(), I2cError> {
        unsafe {
            core::ptr::write_volatile(reg8(REG_I2C_ID), address_byte);
            core::ptr::write_volatile(
                reg8(REG_I2C_CTRL),
                FLD_I2C_CMD_START
                    | if is_read {
                        FLD_I2C_CMD_ID
                    } else {
                        FLD_I2C_CMD_ID
                    },
            );
        }
        let status = self.wait_while_busy()?;
        if (status & FLD_I2C_NAK) != 0 {
            return Err(I2cError::NoAcknowledge(NoAcknowledgeSource::Address));
        }
        Ok(())
    }

    fn write_byte(&self, byte: u8) -> Result<(), I2cError> {
        unsafe {
            core::ptr::write_volatile(reg8(REG_I2C_DO), byte);
            core::ptr::write_volatile(reg8(REG_I2C_CTRL), FLD_I2C_CMD_DO);
        }
        let status = self.wait_while_busy()?;
        if (status & FLD_I2C_NAK) != 0 {
            return Err(I2cError::NoAcknowledge(NoAcknowledgeSource::Data));
        }
        Ok(())
    }

    fn read_byte(&self, last: bool) -> Result<u8, I2cError> {
        let command = FLD_I2C_CMD_DI | FLD_I2C_CMD_READ_ID | if last { FLD_I2C_CMD_ACK } else { 0 };
        unsafe {
            core::ptr::write_volatile(reg8(REG_I2C_CTRL), command);
        }
        let _ = self.wait_while_busy()?;
        Ok(unsafe { core::ptr::read_volatile(reg8(REG_I2C_DI).cast_const()) })
    }

    fn begin_for_operation(
        &self,
        address: SevenBitAddress,
        current_is_read: bool,
        first: bool,
    ) -> Result<(), I2cError> {
        if !first {
            self.reset_bus();
        }
        if current_is_read {
            self.start_read(address)
        } else {
            self.start_write(address)
        }
    }
}

impl ErrorType for I2c {
    type Error = I2cError;
}

impl EhI2c<SevenBitAddress> for I2c {
    fn transaction(
        &mut self,
        address: SevenBitAddress,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        if operations.is_empty() {
            return Ok(());
        }

        let mut result = Ok(());
        let mut previous_is_read: Option<bool> = None;

        for operation in operations.iter_mut() {
            let current_is_read = matches!(operation, Operation::Read(_));
            if previous_is_read != Some(current_is_read) {
                if let Err(err) =
                    self.begin_for_operation(address, current_is_read, previous_is_read.is_none())
                {
                    result = Err(err);
                    break;
                }
                previous_is_read = Some(current_is_read);
            }

            match operation {
                Operation::Write(bytes) => {
                    for &byte in bytes.iter() {
                        if let Err(err) = self.write_byte(byte) {
                            result = Err(err);
                            break;
                        }
                    }
                }
                Operation::Read(buffer) => {
                    let last_index = buffer.len().saturating_sub(1);
                    for (index, byte) in buffer.iter_mut().enumerate() {
                        match self.read_byte(index == last_index) {
                            Ok(value) => *byte = value,
                            Err(err) => {
                                result = Err(err);
                                break;
                            }
                        }
                    }
                }
            }

            if result.is_err() {
                break;
            }
        }

        let stop_result = self.stop();
        match (result, stop_result) {
            (Err(err), _) => Err(err),
            (Ok(()), Err(err)) => Err(err),
            (Ok(()), Ok(())) => Ok(()),
        }
    }
}

fn clamp_frequency(frequency_hz: u32) -> u32 {
    frequency_hz.clamp(1, I2C_MAX_FREQ_HZ)
}

fn enable_clock() {
    unsafe {
        let value = core::ptr::read_volatile(reg8(REG_CLK_EN0).cast_const()) | FLD_CLK0_I2C_EN;
        core::ptr::write_volatile(reg8(REG_CLK_EN0), value);
    }
}

fn disable_spi_pad_mode() {
    unsafe {
        let value = core::ptr::read_volatile(reg8(REG_SPI_SP).cast_const()) & !FLD_SPI_ENABLE;
        core::ptr::write_volatile(reg8(REG_SPI_SP), value);
    }
}

fn configure_gpio(pin_group: I2cPinGroup) {
    unsafe extern "C" {
        fn i2c_gpio_set(i2c_pin_group: u32);
    }

    unsafe {
        i2c_gpio_set(pin_group as u32);
    }
}
