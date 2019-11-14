use crate::common::*;

use failure::Error;
use std::{
    ffi::OsStr,
    iter::once,
    mem::size_of,
    os::windows::ffi::OsStrExt,
    ptr::{null, null_mut},
};
use winapi::{shared::guiddef::GUID, um::setupapi::*};

pub fn _enumerate() -> Vec<USBDevice> {
    let mut output: Vec<USBDevice> = Vec::new();

    let flags = DIGCF_ALLCLASSES | DIGCF_PRESENT;
    let wide: Vec<u16> = OsStr::new("USB").encode_wide().chain(once(0)).collect();

    let dev_info = unsafe { SetupDiGetClassDevsW(null(), wide.as_ptr(), null_mut(), flags) };

    let mut dev_info_data: Vec<SP_DEVINFO_DATA> = vec![SP_DEVINFO_DATA {
        cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
        ClassGuid: GUID::default(),
        DevInst: 0,
        Reserved: 0,
    }];

    let mut i = 0;
    while unsafe { SetupDiEnumDeviceInfo(dev_info, i, dev_info_data.as_mut_ptr()) } > 0 {
        let mut n_size: Vec<u32> = vec![0];
        let mut data_t: Vec<u32> = vec![0];
        let mut buf: Vec<u8> = vec![0; 1000];

        if unsafe {
            SetupDiGetDeviceRegistryPropertyW(
                dev_info,
                dev_info_data.as_mut_ptr(),
                SPDRP_HARDWAREID,
                data_t.as_mut_ptr(),
                buf.as_mut_ptr(),
                buf.len() as u32,
                n_size.as_mut_ptr(),
            )
        } > 0
        {
            if let Ok((vid, pid)) = extract_vid_pid(buf) {
                output.push(USBDevice { vid, pid });
            }
        }

        i += 1;
    }

    output
}

fn extract_vid_pid(buf: Vec<u8>) -> Result<(u16, u16), Error> {
    // Convert to u16 so we can convert from utf16
    let str_vec: Vec<u16> = buf
        .chunks_exact(2)
        .into_iter()
        .map(|a| u16::from_ne_bytes([a[0], a[1]]))
        .collect();

    let id = String::from_utf16(&str_vec)?.to_uppercase();
    let id = id.trim_matches(char::from(0));

    let vid = id.find("VID_").ok_or(ParseError)?;
    let pid = id.find("PID_").ok_or(ParseError)?;

    Ok((
        u16::from_str_radix(&id[vid + 4..vid + 8], 16)?,
        u16::from_str_radix(&id[pid + 4..pid + 8], 16)?,
    ))
}
