//! A cross platform Rust library that returns the vendor and product IDs of
//! currently connected USB devices
//!
//! [![Actions Status](https://github.com/timfish/usb-enumeration/workflows/Build/badge.svg)](https://github.com/timfish/usb-enumeration/actions)
//!
//! # Example
//! ```no_run
//! let devices = usb_enumeration::enumerate(None, None);
//!
//! println!("{:#?}", devices);
//!
//! // Outputs:
//! // [
//! //   UsbDevice {
//! //       id: "USB\\VID_0CE9&PID_1220\\0000000004BE",
//! //       vendor_id: 3305,
//! //       product_id: 4640,
//! //       description: Some(
//! //           "PicoScope 4000 series PC Oscilloscope",
//! //       ),
//! //   },
//! //   UsbDevice {
//! //       id: "USB\\VID_046D&PID_C52B\\5&17411534&0&11",
//! //       vendor_id: 1133,
//! //       product_id: 50475,
//! //       description: Some(
//! //           "USB Composite Device",
//! //       ),
//! //   },
//! //   UsbDevice {
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
//! You can also subscribe to events using the `Observer`:
//! ```no_run
//! use usb_enumeration::{Observer, Event};
//!
//! let sub = Observer::new()
//!     .with_poll_interval(2)
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
pub use common::UsbDevice;
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use std::{collections::HashSet, thread, time::Duration};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use crate::windows::enumerate_platform;

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
/// * `vendor_id` - Optional USB Vendor ID to filter
/// * `product_id` - Optional USB Product ID to filter
///
/// ```no_run
/// let devices = usb_enumeration::enumerate(None, None);
/// ```
/// You can also optionally filter by vendor or product ID:
/// ```no_run
/// let devices = usb_enumeration::enumerate(Some(0x1234), None);
/// ```
#[must_use]
pub fn enumerate(vendor_id: Option<u16>, product_id: Option<u16>) -> Vec<UsbDevice> {
    enumerate_platform(vendor_id, product_id)
}

/// Events send from the Observer
#[derive(Debug, Clone)]
pub enum Event {
    /// Initial list of devices when polling starts
    Initial(Vec<UsbDevice>),
    /// A device that has just been connected
    Connect(UsbDevice),
    /// A device that has just disconnected
    Disconnect(UsbDevice),
}

#[derive(Clone)]
pub struct Subscription {
    pub rx_event: Receiver<Event>,
    // When this gets dropped, the channel will become disconnected and the
    // background thread will close
    _tx_close: Sender<()>,
}

#[derive(Debug, Clone)]
pub struct Observer {
    poll_interval: u32,
    vendor_id: Option<u16>,
    product_id: Option<u16>,
}

impl Default for Observer {
    fn default() -> Self {
        Observer::new()
    }
}

impl Observer {
    /// Create a new Observer with the poll interval specified in seconds
    pub fn new() -> Self {
        Observer {
            poll_interval: 1,
            vendor_id: None,
            product_id: None,
        }
    }

    pub fn with_poll_interval(mut self, seconds: u32) -> Self {
        self.poll_interval = seconds;
        self
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

    /// Start the background thread and poll for device changes
    pub fn subscribe(&self) -> Subscription {
        let (tx_event, rx_event) = unbounded();
        let (tx_close, rx_close) = bounded::<()>(0);

        thread::Builder::new()
            .name("USB Enumeration Thread".to_string())
            .spawn({
                let this = self.clone();
                move || {
                    let device_list = enumerate(this.vendor_id, this.product_id);

                    // Send initially connected devices
                    if tx_event.send(Event::Initial(device_list.clone())).is_err() {
                        return;
                    }

                    let mut device_list: HashSet<UsbDevice> = device_list.into_iter().collect();
                    let mut wait_seconds = this.poll_interval as f32;

                    loop {
                        while wait_seconds > 0.0 {
                            // Check whether the subscription has been disposed
                            if let Err(crossbeam::channel::RecvTimeoutError::Disconnected) =
                                rx_close.recv_timeout(Duration::from_millis(250))
                            {
                                return;
                            }

                            wait_seconds -= 0.25;
                        }

                        wait_seconds = this.poll_interval as f32;

                        let next_devices: HashSet<UsbDevice> =
                            enumerate(this.vendor_id, this.product_id)
                                .into_iter()
                                .collect();

                        // Send Disconnect for missing devices
                        for device in &device_list {
                            if !next_devices.contains(device)
                                && tx_event.send(Event::Disconnect(device.clone())).is_err()
                            {
                                return;
                            }
                        }

                        // Send Connect for new devices
                        for device in &next_devices {
                            if !device_list.contains(device)
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

        Subscription {
            rx_event,
            _tx_close: tx_close,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate() {
        let devices = enumerate(None, None);
        println!("Enumerated devices: {devices:#?}");
        assert!(!devices.is_empty());
    }

    #[test]
    fn test_subscribe() {
        let subscription = Observer::new().subscribe();
        let mut iter = subscription.rx_event.iter();

        let initial = iter.next().expect("Should get an Event");
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
