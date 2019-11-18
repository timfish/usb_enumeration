use crate::common::*;
use core_foundation::{base::*, dictionary::*, number::*, string::*};
use failure::Error;
use io_kit_sys::{types::*, usb::lib::*, *};
use mach::kern_return::*;
use std::mem::MaybeUninit;

pub fn _enumerate() -> Vec<USBDevice> {
    let mut output = Vec::new();

    unsafe {
        let matching_dict = IOServiceMatching(kIOUSBDeviceClassName);
        if let None = matching_dict.as_ref() {
            panic!("Failed to get IOServiceMatching");
        }

        let mut iter: io_iterator_t = MaybeUninit::uninit().assume_init();

        let kr = IOServiceGetMatchingServices(kIOMasterPortDefault, matching_dict, &mut iter);
        if kr != KERN_SUCCESS {
            panic!("Failed IOServiceGetMatchingServices");
        }

        #[allow(unused_assignments)]
        let mut device: io_service_t = MaybeUninit::uninit().assume_init();

        while (device = IOIteratorNext(iter)) == () && device > 0 {
            let mut props: CFMutableDictionaryRef = MaybeUninit::uninit().assume_init();

            let _result =
                IORegistryEntryCreateCFProperties(device, &mut props, kCFAllocatorDefault, 0);

            let properties: CFDictionary<CFString, CFType> =
                CFMutableDictionary::wrap_under_get_rule(props).to_immutable();

            if let Ok((vid, pid)) = get_vid_pid(properties) {
                output.push(USBDevice { vid, pid });
            }

            IOObjectRelease(device);
        }

        IOObjectRelease(iter);
    }

    output
}

fn get_vid_pid(properties: CFDictionary<CFString, CFType>) -> Result<(u16, u16), Error> {
    let key = CFString::from_static_string("idVendor");

    let vid = properties
        .find(&key)
        .and_then(|value_ref| value_ref.downcast::<CFNumber>())
        .ok_or(ParseError)?
        .to_i32()
        .ok_or(ParseError)? as u16;

    let key = CFString::from_static_string("idProduct");

    let pid = properties
        .find(&key)
        .and_then(|value_ref| value_ref.downcast::<CFNumber>())
        .ok_or(ParseError)?
        .to_i32()
        .ok_or(ParseError)? as u16;

    Ok((vid, pid))
}
