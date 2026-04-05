#![no_std]

//! Peripheral access crate for TLSR82xx targets.
//!
//! Select a target chip with one of:
//! - `chip-8258`
//! - `chip-8278`
//! - `chip-826x`

#[cfg(all(feature = "chip-8258", feature = "chip-8278"))]
compile_error!("enable only one tlsr82xx-pac chip feature at a time");
#[cfg(all(feature = "chip-8258", feature = "chip-826x"))]
compile_error!("enable only one tlsr82xx-pac chip feature at a time");
#[cfg(all(feature = "chip-8278", feature = "chip-826x"))]
compile_error!("enable only one tlsr82xx-pac chip feature at a time");

#[cfg(feature = "chip-8258")]
pub use tlsr82xx_pac_8258::*;

#[cfg(feature = "chip-8278")]
pub use tlsr82xx_pac_8278::*;

#[cfg(feature = "chip-826x")]
pub use tlsr82xx_pac_826x::*;

#[cfg(not(any(feature = "chip-8258", feature = "chip-8278", feature = "chip-826x")))]
mod placeholder {
    //! No chip feature selected.
}
