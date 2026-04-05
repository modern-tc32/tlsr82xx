use core::fmt;

use crate::pac;

const RESET_BASE: usize = 0x0080_0060;
const UART_BASE: usize = 0x0080_0090;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum StopBits {
    One = 0x00,
    OneAndHalf = 0x10,
    Two = 0x20,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    pub baudrate: u32,
    pub sysclk_hz: u32,
    pub parity: Parity,
    pub stop_bits: StopBits,
}

impl Config {
    pub const fn new(baudrate: u32, sysclk_hz: u32) -> Self {
        Self {
            baudrate,
            sysclk_hz,
            parity: Parity::None,
            stop_bits: StopBits::One,
        }
    }

    pub const fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    pub const fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }
}

pub trait UartExt {
    fn constrain(self) -> Uart;
}

impl UartExt for pac::Uart {
    fn constrain(self) -> Uart {
        Uart::new(self)
    }
}

pub struct Uart {
    _inner: pac::Uart,
    tx_index: u8,
}

impl Uart {
    pub fn new(inner: pac::Uart) -> Self {
        let uart = Self {
            _inner: inner,
            tx_index: 0,
        };
        uart.enable_peripheral();
        uart
    }

    #[inline(always)]
    fn reg8(offset: usize) -> *mut u8 {
        (UART_BASE + offset) as *mut u8
    }

    #[inline(always)]
    fn reg16(offset: usize) -> *mut u16 {
        (UART_BASE + offset) as *mut u16
    }

    #[inline(always)]
    fn enable_peripheral(&self) {
        unsafe {
            let clk_en0 = (RESET_BASE + 0x03) as *mut u8;
            let rst0 = RESET_BASE as *mut u8;

            core::ptr::write_volatile(
                clk_en0,
                core::ptr::read_volatile(clk_en0.cast_const()) | (1 << 2),
            );
            core::ptr::write_volatile(rst0, core::ptr::read_volatile(rst0.cast_const()) | (1 << 2));
            core::ptr::write_volatile(
                rst0,
                core::ptr::read_volatile(rst0.cast_const()) & !(1 << 2),
            );
        }
    }

    pub fn configure(&mut self, config: Config) {
        let (div, bwpc) = compute_baud_params(config.sysclk_hz, config.baudrate);

        unsafe {
            // ctrl0: keep DMA/IRQ disabled, update bit width per clock.
            let ctrl0 = Self::reg8(0x06);
            let mut ctrl0_value = core::ptr::read_volatile(ctrl0.cast_const()) & !0x0f;
            ctrl0_value |= bwpc & 0x0f;
            core::ptr::write_volatile(ctrl0, ctrl0_value);

            // clk_div: 15-bit divider plus enable bit.
            core::ptr::write_volatile(Self::reg16(0x04), div | 0x8000);

            // timeout = 3 * (bwpc + 1), multiplier fixed to SDK default.
            core::ptr::write_volatile(Self::reg8(0x0a), (bwpc.wrapping_add(1)).wrapping_mul(3));
            let timeout1 = Self::reg8(0x0b);
            let mut timeout1_value = core::ptr::read_volatile(timeout1.cast_const()) & !0x03;
            timeout1_value |= 0x01;
            core::ptr::write_volatile(timeout1, timeout1_value);

            let ctrl1 = Self::reg8(0x07);
            let mut ctrl1_value = core::ptr::read_volatile(ctrl1.cast_const()) & !0x3c;
            ctrl1_value = match config.parity {
                Parity::None => ctrl1_value & !(1 << 2),
                Parity::Even => (ctrl1_value | (1 << 2)) & !(1 << 3),
                Parity::Odd => ctrl1_value | (1 << 2) | (1 << 3),
            };
            ctrl1_value |= config.stop_bits as u8;
            core::ptr::write_volatile(ctrl1, ctrl1_value);
        }

        self.tx_index = 0;
    }

    pub fn is_tx_busy(&self) -> bool {
        unsafe { (core::ptr::read_volatile(Self::reg8(0x0e).cast_const()) & 0x01) == 0 }
    }

    pub fn flush(&mut self) {
        while self.is_tx_busy() {
            core::hint::spin_loop();
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        while self.tx_fifo_count() > 7 {
            core::hint::spin_loop();
        }

        unsafe {
            core::ptr::write_volatile(Self::reg8(self.tx_index as usize), byte);
        }
        self.tx_index = (self.tx_index + 1) & 0x03;
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
    }

    #[inline(always)]
    fn tx_fifo_count(&self) -> u8 {
        unsafe { core::ptr::read_volatile(Self::reg8(0x0c).cast_const()) >> 4 }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}

fn compute_baud_params(sysclk_hz: u32, baudrate: u32) -> (u16, u8) {
    assert!(sysclk_hz != 0, "sysclk_hz must not be zero");
    assert!(baudrate != 0, "baudrate must not be zero");

    let mut best_div = 0u16;
    let mut best_bwpc = 0u8;
    let mut best_error = u32::MAX;

    for bwpc in 0u8..=15 {
        let denom = baudrate.saturating_mul(u32::from(bwpc) + 1);
        if denom == 0 {
            continue;
        }

        let div = sysclk_hz / denom;
        if div == 0 {
            continue;
        }

        let div = div - 1;
        if div > 0x7fff {
            continue;
        }

        let actual = sysclk_hz / ((div + 1) * (u32::from(bwpc) + 1));
        let error = actual.abs_diff(baudrate);
        if error < best_error {
            best_error = error;
            best_div = div as u16;
            best_bwpc = bwpc;
        }
    }

    assert!(
        best_error != u32::MAX,
        "unable to derive UART baud configuration"
    );
    (best_div, best_bwpc)
}
