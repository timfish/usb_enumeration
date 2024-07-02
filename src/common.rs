use num_enum::TryFromPrimitive;
use std::error::Error;

/// Discovered USB device
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UsbDevice {
    /// Platform specific unique ID
    pub id: String,
    /// Vendor ID
    pub vendor_id: u16,
    /// Product ID
    pub product_id: u16,
    /// Optional device description
    pub description: Option<String>,
    /// Optional serial number
    pub serial_number: Option<String>,
    /// Class of device.
    pub base_class: DeviceBaseClass,
}

/// See <https://www.usb.org/defined-class-codes>
#[repr(u8)]
#[derive(Hash, Eq, Debug, Clone, PartialEq, TryFromPrimitive)]
pub enum DeviceBaseClass {
    UseClassCodeFromInterfaceDescriptors = 0x00,
    Audio = 0x01,
    Communication = 0x02,
    HumanInterfaceDevice = 0x03,
    Physical = 0x05,
    Image = 0x06,
    Printer = 0x07,
    MassStorage = 0x08,
    Hub = 0x09,
    CDCData = 0x0A,
    SmartCard = 0x0B,
    ContentSecurity = 0x0D,
    Video = 0x0E,
    PersonalHealthCare = 0x0F,
    AudioVideo = 0x10,
    Billboard = 0x11,
    UsbTypeCBridge = 0x12,
    UsbBulkDisplay = 0x13,
    MctpOverUsb = 0x14,
    I3C = 0x3C,
    Diagnostic = 0xDC,
    WirelessController = 0xE0,
    Miscellaneous = 0xEF,
    ApplicationSpecific = 0xFE,
    VendorSpecific = 0xFF,
}

#[derive(Copy, Clone, Debug)]
pub struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse Error")?;
        Ok(())
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
