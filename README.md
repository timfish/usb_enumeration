[![Actions Status](https://github.com/timfish/usb-enumeration/workflows/Build/badge.svg)](https://github.com/timfish/usb-enumeration/actions)

A cross platform Rust library that returns the vendor and product IDs of
currently connected USB devices

Requires Rust >= 1.36.0

```rust
let devices = usb_enumeration::enumerate();

println!("{:#?}", devices);

// Outputs:
// [
//     USBDevice {
//         vid: 1118,
//         pid: 1957,
//     },
//     USBDevice {
//         vid: 1118,
//         pid: 1957,
//     },
//     etc...
// ]
```