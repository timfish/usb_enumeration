//! Enumerate usb devices.

fn main() {
    let devices = usb_enumeration::enumerate(None, None);
    println!("{:#?}", devices);
}
