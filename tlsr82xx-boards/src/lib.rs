#![no_std]

#[cfg(all(feature = "chip-8258", feature = "chip-8278"))]
compile_error!("enable only one tlsr82xx-boards chip feature at a time");
#[cfg(all(feature = "chip-8258", feature = "chip-826x"))]
compile_error!("enable only one tlsr82xx-boards chip feature at a time");
#[cfg(all(feature = "chip-8278", feature = "chip-826x"))]
compile_error!("enable only one tlsr82xx-boards chip feature at a time");
#[cfg(not(any(feature = "chip-8258", feature = "chip-8278", feature = "chip-826x")))]
compile_error!("enable one tlsr82xx-boards chip feature");

pub mod tb03f;
