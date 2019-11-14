use crate::common::*;
use failure::Error;

use udev::{Context, Device, Enumerator};

pub fn _enumerate() -> Vec<USBDevice> {
    let mut output = Vec::new();

    let context = Context::new().expect("could not get udev context");
    let mut enumerator = Enumerator::new(&context).expect("could not get udev enumerator");

    for device in enumerator.scan_devices().expect("could not scan devices") {
        if let Ok((vid, pid)) = get_vid_pid(device) {
            output.push(USBDevice { vid, pid });
        }
    }

    output
}

fn get_vid_pid(device: Device) -> Result<(u16, u16), Error> {
    let vid = device.property_value("ID_VENDOR_ID").ok_or(ParseError)?;
    let pid = device.property_value("ID_MODEL_ID").ok_or(ParseError)?;

    let mut vid = vid.to_str().ok_or(ParseError)?;
    let mut pid = pid.to_str().ok_or(ParseError)?;

    // Sometimes they are prefixed
    if vid.starts_with("0x") {
        vid = &vid[2..];
    }

    if pid.starts_with("0x") {
        pid = &pid[2..];
    }

    return Ok((
        u16::from_str_radix(&vid, 16)?,
        u16::from_str_radix(&pid, 16)?,
    ));
}
