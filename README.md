# usb_enumeration

A cross platform Rust library that returns the vendor and product IDs of
currently connected USB devices

[![Actions Status](https://github.com/timfish/usb-enumeration/workflows/Build/badge.svg)](https://github.com/timfish/usb-enumeration/actions)

## Example

```rust
let devices = usb_enumeration::enumerate(None, None);

println!("{:#?}", devices);

// Outputs:
// [
//   UsbDevice {
//       id: "USB\\VID_0CE9&PID_1220\\0000000004BE",
//       vendor_id: 3305,
//       product_id: 4640,
//       description: Some(
//           "PicoScope 4000 series PC Oscilloscope",
//       ),
//   },
//   UsbDevice {
//       id: "USB\\VID_046D&PID_C52B\\5&17411534&0&11",
//       vendor_id: 1133,
//       product_id: 50475,
//       description: Some(
//           "USB Composite Device",
//       ),
//   },
//   UsbDevice {
//       id: "USB\\VID_046D&PID_C52B&MI_00\\6&12D311A2&0&0000",
//       vendor_id: 1133,
//       product_id: 50475,
//       description: Some(
//           "Logitech USB Input Device",
//       ),
//   },
//     etc...
// ]
```

You can also subscribe to events using the `Observer`:

```rust
use usb_enumeration::{Observer, Event};

let sub = Observer::new()
    .with_poll_interval(2)
    .with_vendor_id(0x1234)
    .with_product_id(0x5678)
    .subscribe();

// when sub is dropped, the background thread will close

for event in sub.rx_event.iter() {
    match event {
        Event::Initial(d) => println!("Initial devices: {:?}", d),
        Event::Connect(d) => println!("Connected device: {:?}", d),
        Event::Disconnect(d) => println!("Disconnected device: {:?}", d),
    }
}
```

License: MIT
