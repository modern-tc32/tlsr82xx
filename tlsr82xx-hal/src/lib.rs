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

#[macro_export]
macro_rules! ram_irq_handler {
    ($handler:path) => {{
        unsafe { $crate::interrupt::RamIrqHandler::__new($handler) }
    }};
}

#[macro_export]
macro_rules! define_ram_irq_handler {
    ($(#[$meta:meta])* $vis:vis unsafe extern "C" fn $name:ident ($arg:ident : u32) $body:block) => {
        $(#[$meta])*
        #[unsafe(link_section = ".ram_code")]
        $vis unsafe extern "C" fn $name($arg: u32) $body
    };
}

#[macro_export]
macro_rules! ram_global_irq_handler {
    ($handler:path) => {{
        unsafe { $crate::interrupt::RamGlobalIrqHandler::__new($handler) }
    }};
}

#[macro_export]
macro_rules! define_ram_global_irq_handler {
    ($(#[$meta:meta])* $vis:vis unsafe extern "C" fn $name:ident () $body:block) => {
        $(#[$meta])*
        #[unsafe(link_section = ".ram_code")]
        $vis unsafe extern "C" fn $name() $body
    };
}

#[macro_export]
macro_rules! ram_void_handler {
    ($handler:path) => {{
        unsafe { $crate::interrupt::RamVoidHandler::__new($handler) }
    }};
}

#[macro_export]
macro_rules! define_ram_void_handler {
    ($(#[$meta:meta])* $vis:vis unsafe extern "C" fn $name:ident () $body:block) => {
        $(#[$meta])*
        #[unsafe(link_section = ".ram_code")]
        $vis unsafe extern "C" fn $name() $body
    };
}

#[cfg(feature = "chip-8258")]
#[macro_export]
macro_rules! ram_rf_irq_handler {
    ($handler:path) => {{
        unsafe { $crate::interrupt::RamRfIrqHandler::__new($handler) }
    }};
}

#[cfg(feature = "chip-8258")]
#[macro_export]
macro_rules! define_ram_rf_irq_handler {
    ($(#[$meta:meta])* $vis:vis unsafe extern "C" fn $name:ident ($arg:ident : u16) $body:block) => {
        $(#[$meta])*
        #[unsafe(link_section = ".ram_code")]
        $vis unsafe extern "C" fn $name($arg: u16) $body
    };
}
