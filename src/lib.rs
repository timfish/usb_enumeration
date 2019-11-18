//! A cross platform Rust library that returns the vendor and product IDs of
//! currently connected USB devices
//!
//! # Example
//! ```
//! let devices = usb_enumeration::enumerate();
//!
//! println!("{:#?}", devices);
//!
//! // Outputs:
//! // [
//! //     USBDevice {
//! //         vid: 1118,
//! //         pid: 1957,
//! //     },
//! //     USBDevice {
//! //         vid: 1118,
//! //         pid: 1957,
//! //     },
//! //     etc...
//! // ]
//! ```

#![cfg_attr(feature = "strict", deny(warnings))]

#[macro_use]
extern crate failure;

mod common;
pub use common::USBDevice;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use crate::windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::macos::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use crate::linux::*;

/// Enumerates connected USB devices
///
/// ```
/// let devices = usb_enumeration::enumerate();
/// ```
pub fn enumerate() -> Vec<USBDevice> {
    _enumerate()
}

#[cfg(test)]
mod tests {
    // run `cargo test -- --nocapture` to see connected devices
    use super::*;

    #[test]
    fn test_enumerate() {
        let devices = enumerate();
        println!("{:#?}", devices);
        assert!(devices.len() > 0);
    }
}
