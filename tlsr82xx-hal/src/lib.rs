#![no_std]

//! Hardware abstraction layer for TLSR82xx targets.

#[cfg(all(feature = "chip-8258", feature = "chip-8278"))]
compile_error!("enable only one tlsr82xx-hal chip feature at a time");
#[cfg(all(feature = "chip-8258", feature = "chip-826x"))]
compile_error!("enable only one tlsr82xx-hal chip feature at a time");
#[cfg(all(feature = "chip-8278", feature = "chip-826x"))]
compile_error!("enable only one tlsr82xx-hal chip feature at a time");
#[cfg(not(any(feature = "chip-8258", feature = "chip-8278", feature = "chip-826x")))]
compile_error!("enable one tlsr82xx-hal chip feature");
#[cfg(all(feature = "irq-timer0-shim", not(feature = "chip-8258")))]
compile_error!("irq-timer0-shim is only supported on chip-8258");
#[cfg(all(feature = "irq-system-timer-shim", not(feature = "chip-8258")))]
compile_error!("irq-system-timer-shim is only supported on chip-8258");

pub use tlsr82xx_pac as pac;

#[cfg(feature = "chip-8258")]
pub mod adc;
pub mod analog;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod clock;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod flash;
pub mod gpio;
#[cfg(feature = "chip-8258")]
pub mod i2c;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod interrupt;
#[cfg(any(feature = "chip-8258", feature = "chip-8278", feature = "chip-826x"))]
pub mod pm;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod pwm;
#[cfg(feature = "chip-8258")]
pub mod radio;
#[cfg(feature = "chip-8258")]
pub mod spi;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod startup;
pub mod timer;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod uart;

mod mmio;
#[cfg(feature = "chip-8258")]
mod regs8258;
