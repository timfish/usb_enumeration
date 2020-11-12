//! A cross platform Rust library that returns the vendor and product IDs of
//! currently connected USB devices
//!
//! [![Actions Status](https://github.com/timfish/usb-enumeration/workflows/Build/badge.svg)](https://github.com/timfish/usb-enumeration/actions)
//!
//! # Example
//! ```
//! let devices = usb_enumeration::enumerate();
//!
//! println!("{:#?}", devices);
//!
//! // Outputs:
//! // [
//! //   USBDevice {
//! //       id: "USB\\VID_0CE9&PID_1220\\0000000004BE",
//! //       vendor_id: 3305,
//! //       product_id: 4640,
//! //       description: Some(
//! //           "PicoScope 4000 series PC Oscilloscope",
//! //       ),
//! //   },
//! //   USBDevice {
//! //       id: "USB\\VID_046D&PID_C52B\\5&17411534&0&11",
//! //       vendor_id: 1133,
//! //       product_id: 50475,
//! //       description: Some(
//! //           "USB Composite Device",
//! //       ),
//! //   },
//! //   USBDevice {
//! //       id: "USB\\VID_046D&PID_C52B&MI_00\\6&12D311A2&0&0000",
//! //       vendor_id: 1133,
//! //       product_id: 50475,
//! //       description: Some(
//! //           "Logitech USB Input Device",
//! //       ),
//! //   },
//! //     etc...
//! // ]
//! ```
//! You can also subscribe events using the `Observer`:
//! ```no_run
//! use usb_enumeration::{Observer, Event};
//!
//! // Set the poll interval to 2 seconds
//! let sub = Observer::new(2)
//!     .with_vendor_id(0x1234)
//!     .with_product_id(0x5678)
//!     .subscribe();
//!
//! // when sub is dropped, the background thread will close
//!
//! for event in sub.rx_event.iter() {
//!     match event {
//!         Event::Initial(d) => println!("Initial devices: {:?}", d),
//!         Event::Connect(d) => println!("Connected device: {:?}", d),
//!         Event::Disconnect(d) => println!("Disconnected device: {:?}", d),
//!     }   
//! }
//! ```

#![cfg_attr(feature = "strict", deny(warnings))]

mod common;
pub use common::USBDevice;
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use std::{collections::HashSet, thread, time::Duration};

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

/// # Enumerates connected USB devices
///
/// * `vendor_id` - USB Vendor ID to filter
/// * `product_id` - USB Product ID to filter
///
/// ```
/// let devices = usb_enumeration::enumerate();
/// ```
/// There are also some handy filters:
/// ```
/// use usb_enumeration::Filters;
///
/// let devices = usb_enumeration::enumerate().with_vendor_id(0x1234);
/// ```
pub fn enumerate() -> Vec<USBDevice> {
    enumerate_platform()
}

pub trait Filters {
    fn with_vendor_id(self, vendor_id: u16) -> Vec<USBDevice>;
    fn with_product_id(self, product_id: u16) -> Vec<USBDevice>;
}

impl Filters for Vec<USBDevice> {
    fn with_vendor_id(self, vendor_id: u16) -> Vec<USBDevice> {
        self.into_iter()
            .filter(|d| d.vendor_id == vendor_id)
            .collect()
    }

    fn with_product_id(self, product_id: u16) -> Vec<USBDevice> {
        self.into_iter()
            .filter(|d| d.product_id == product_id)
            .collect()
    }
}

/// Events send from the Observer
#[derive(Debug, Clone)]
pub enum Event {
    /// Initial list of devices when polling starts
    Initial(Vec<USBDevice>),
    /// A device that has just been connected
    Connect(USBDevice),
    /// A device that has just disconnected
    Disconnect(USBDevice),
}

#[derive(Clone)]
pub struct Subscription {
    pub rx_event: Receiver<Event>,
    // When this gets dropped, the channel will become disconnected and the
    // background thread will close
    tx_close: Sender<()>,
}

#[derive(Debug, Clone)]
pub struct Observer {
    poll_interval: u64,
    vendor_id: Option<u16>,
    product_id: Option<u16>,
}

impl Observer {
    /// Create a new Observer with the poll interval specified in seconds
    pub fn new(poll_interval: u64) -> Self {
        Observer {
            poll_interval,
            vendor_id: None,
            product_id: None,
        }
    }

    /// Filter results by USB Vendor ID
    pub fn with_vendor_id(mut self, vendor_id: u16) -> Self {
        self.vendor_id = Some(vendor_id);
        self
    }

    /// Filter results by USB Product ID
    pub fn with_product_id(mut self, product_id: u16) -> Self {
        self.product_id = Some(product_id);
        self
    }

    fn enumerate(&self) -> Vec<USBDevice> {
        let mut devices = enumerate();

        if let Some(vendor_id) = self.vendor_id {
            devices = devices.with_vendor_id(vendor_id);
        }

        if let Some(product_id) = self.product_id {
            devices = devices.with_product_id(product_id);
        }

        devices
    }

    /// Start the background thread and poll for device changes
    pub fn subscribe(&self) -> Subscription {
        let (tx_event, rx_event) = unbounded();
        let (tx_close, rx_close) = bounded::<()>(0);

        thread::Builder::new()
            .name("USB Enumeration Thread".to_string())
            .spawn({
                let this = self.clone();
                move || {
                    let device_list = this.enumerate();

                    // Send initially connected devices
                    if tx_event.send(Event::Initial(device_list.clone())).is_err() {
                        return;
                    }

                    let mut device_list: HashSet<USBDevice> = device_list.into_iter().collect();
                    let mut wait_seconds = this.poll_interval;

                    loop {
                        while wait_seconds > 0 {
                            // Check whether the subscription has been disposed
                            if let Err(crossbeam::channel::RecvTimeoutError::Disconnected) =
                                rx_close.recv_timeout(Duration::from_secs(1))
                            {
                                return;
                            }

                            wait_seconds -= 1;
                        }

                        wait_seconds = this.poll_interval;

                        let next_devices: HashSet<USBDevice> =
                            this.enumerate().into_iter().collect();

                        // Send Disconnect for missing devices
                        for device in &device_list {
                            if !next_devices.contains(&device)
                                && tx_event.send(Event::Disconnect(device.clone())).is_err()
                            {
                                return;
                            }
                        }

                        // Send Connect for new devices
                        for device in &next_devices {
                            if !device_list.contains(&device)
                                && tx_event.send(Event::Connect(device.clone())).is_err()
                            {
                                return;
                            }
                        }

                        device_list = next_devices;
                    }
                }
            })
            .expect("Could not spawn background thread");

        Subscription { rx_event, tx_close }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate() {
        let devices = enumerate();
        println!("Enumerated devices: {:#?}", devices);
        assert!(!devices.is_empty());
    }

    #[test]
    fn test_subscribe() {
        let subscription = Observer::new(1).subscribe();
        let mut iter = subscription.rx_event.iter();

        let initial = iter.next().unwrap();
        assert!(matches!(initial, Event::Initial(_)));

        println!("Connect a USB device");

        let connect_event = iter.next().expect("Should get an Event");
        let connect_device = if let Event::Connect(device) = connect_event {
            device
        } else {
            panic!("Expected Event::Connect. Actual: {:?}", connect_event);
        };

        println!("Disconnect that same device");

        let disconnect_event = iter.next().expect("Should get an Event");
        let disconnect_device = if let Event::Disconnect(device) = disconnect_event {
            device
        } else {
            panic!("Expected Event::Disconnect. Actual: {:?}", disconnect_event);
        };

        assert_eq!(connect_device, disconnect_device);
    }
}
