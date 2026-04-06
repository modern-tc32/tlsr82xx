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

pub use tlsr82xx_pac as pac;

pub mod analog;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod clock;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod flash;
pub mod gpio;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod interrupt;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod pwm;
#[cfg(feature = "chip-8258")]
pub mod radio;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod startup;
pub mod timer;
#[cfg(any(feature = "chip-8258", feature = "chip-8278"))]
pub mod uart;

#[cfg(feature = "chip-8258")]
mod regs8258;
mod mmio;
