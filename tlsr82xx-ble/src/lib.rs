#![no_std]

#[cfg(not(feature = "chip-8258"))]
compile_error!("tlsr82xx-ble currently supports only feature chip-8258");

mod beacon;

pub use beacon::{
    BeaconAdvertiser, BeaconConfig, BeaconError, BeaconEventResult, BeaconFailureReason,
};
