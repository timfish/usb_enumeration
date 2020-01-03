use thiserror::Error;

#[derive(Debug, Clone)]
pub struct USBDevice {
    pub vid: u16,
    pub pid: u16,
}

#[derive(Copy, Clone, Debug, Error)]
#[error("Parse Error")]
pub struct ParseError;
