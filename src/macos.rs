use crate::common::*;
use core_foundation::{base::*, dictionary::*, number::*, string::*};
use io_kit_sys::{types::*, usb::lib::*, *};
use mach::kern_return::*;
use std::{error::Error, mem::MaybeUninit};

pub fn enumerate_platform(vid: Option<u16>, pid: Option<u16>) -> Vec<USBDevice> {
    let mut output = Vec::new();

    unsafe {
        let matching_dict = IOServiceMatching(kIOUSBDeviceClassName);
        if matching_dict.as_ref().is_none() {
            panic!("Failed to get IOServiceMatching");
        }

        let mut iter: io_iterator_t = 0;

        let kr = IOServiceGetMatchingServices(kIOMasterPortDefault, matching_dict, &mut iter);
        if kr != KERN_SUCCESS {
            panic!("Failed IOServiceGetMatchingServices");
        }

        #[allow(unused_assignments)]
        let mut device: io_service_t = 0;

        #[allow(clippy::unit_cmp)]
        while (device = IOIteratorNext(iter)) == () && device > 0 {
            #[allow(clippy::uninit_assumed_init)]
            let mut props: CFMutableDictionaryRef = MaybeUninit::uninit().assume_init();

            let _result =
                IORegistryEntryCreateCFProperties(device, &mut props, kCFAllocatorDefault, 0);

            let properties: CFDictionary<CFString, CFType> =
                CFMutableDictionary::wrap_under_get_rule(props).to_immutable();

            let _ = || -> Result<(), Box<dyn Error>> {
                let key = CFString::from_static_string("idVendor");
                let vendor_id = properties
                    .find(&key)
                    .and_then(|value_ref| value_ref.downcast::<CFNumber>())
                    .ok_or(ParseError)?
                    .to_i32()
                    .ok_or(ParseError)? as u16;

                if let Some(vid) = vid {
                    if vid != vendor_id {
                        continue;
                    }
                }

                let key = CFString::from_static_string("idProduct");
                let product_id = properties
                    .find(&key)
                    .and_then(|value_ref| value_ref.downcast::<CFNumber>())
                    .ok_or(ParseError)?
                    .to_i32()
                    .ok_or(ParseError)? as u16;

                if let Some(pid) = pid {
                    if pid != product_id {
                        continue;
                    }
                }

                let key = CFString::from_static_string("sessionID");
                let id = properties
                    .find(&key)
                    .and_then(|value_ref| value_ref.downcast::<CFNumber>())
                    .ok_or(ParseError)?
                    .to_i64()
                    .ok_or(ParseError)?;

                let key = CFString::from_static_string("USB Product Name");
                let description = properties
                    .find(&key)
                    .and_then(|value_ref| value_ref.downcast::<CFString>())
                    .map(|s| s.to_string());

                output.push(USBDevice {
                    id: id.to_string(),
                    vendor_id,
                    product_id,
                    description,
                });

                Ok(())
            }();

            IOObjectRelease(device);
        }

        IOObjectRelease(iter);
    }

    output
}
