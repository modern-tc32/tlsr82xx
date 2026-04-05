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

pub mod gpio;
