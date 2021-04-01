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
