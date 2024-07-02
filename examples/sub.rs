use usb_enumeration::{Event, Observer};

fn main() {
    let sub = Observer::new().with_poll_interval(2).subscribe();

    // when sub is dropped, the background thread will close
    for event in sub.rx_event.iter() {
        match event {
            Event::Initial(d) => println!("Initial devices: {:?}", d),
            Event::Connect(d) => println!("Connected device: {:?}", d),
            Event::Disconnect(d) => println!("Disconnected device: {:?}", d),
        }
    }
}
