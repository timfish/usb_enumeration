use crate::common::*;
use core_foundation::{base::*, dictionary::*, number::*, string::*};
use io_kit_sys::{types::*, usb::lib::*, *};
use mach::kern_return::*;
use std::{error::Error, mem::MaybeUninit};
use std::convert::TryFrom;

pub fn enumerate_platform(vid: Option<u16>, pid: Option<u16>) -> Vec<UsbDevice> {
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
            let mut props = MaybeUninit::<CFMutableDictionaryRef>::uninit();

            let _result = IORegistryEntryCreateCFProperties(
                device,
                props.as_mut_ptr(),
                kCFAllocatorDefault,
                0,
            );

            let props = props.assume_init();

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
                        return Ok(());
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
                        return Ok(());
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

                let key = CFString::from_static_string("USB Serial Number");
                let serial_number = properties
                    .find(&key)
                    .and_then(|value_ref| value_ref.downcast::<CFString>())
                    .map(|s| s.to_string());

                let key = CFString::from_static_string("bDeviceClass");
                let base_class = properties
                    .find(&key)
                    .and_then(|value_ref| value_ref.downcast::<CFNumber>())
                    .ok_or(ParseError)?
                    .to_i32()
                    .ok_or(ParseError)? as u8;

                output.push(UsbDevice {
                    id: id.to_string(),
                    vendor_id,
                    product_id,
                    description,
                    serial_number,
                    base_class: DeviceBaseClass::try_from(base_class)?,
                });

                Ok(())
            }();

            IOObjectRelease(device);
        }

        IOObjectRelease(iter);
    }

    output
}
