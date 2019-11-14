A cross platform Rust library that returns the vendor and product IDs of
currently connected USB devices

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