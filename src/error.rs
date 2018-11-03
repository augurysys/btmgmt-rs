use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    SocketError,
    BindError,
    NoResponse,
    UnknownCommand,
    NotConnected,
    Failed,
    ConnectFailed,
    AuthenticationFailed,
    NotPaired,
    NoResources,
    Timeout,
    AlreadyConnected,
    Busy,
    Rejected,
    NotSupported,
    InvalidParameters,
    Disconnected,
    NotPowered,
    Canceled,
    InvalidIndex,
    RFKilled,
    AlreadyPaired,
    PermissionDenied,
    UnknownError,
}

impl Error {
    pub fn from_status(status: u8) -> Option<Error> {
        match status {
            0x00 => None,
            0x01 => Some(Error::UnknownCommand),
            0x02 => Some(Error::NotConnected),
            0x03 => Some(Error::Failed),
            0x04 => Some(Error::ConnectFailed),
            0x05 => Some(Error::AuthenticationFailed),
            0x06 => Some(Error::NotPaired),
            0x07 => Some(Error::NoResources),
            0x08 => Some(Error::Timeout),
            0x09 => Some(Error::AlreadyConnected),
            0x0A => Some(Error::Busy),
            0x0B => Some(Error::Rejected),
            0x0C => Some(Error::NotSupported),
            0x0D => Some(Error::InvalidParameters),
            0x0E => Some(Error::Disconnected),
            0x0F => Some(Error::NotPowered),
            0x10 => Some(Error::Canceled),
            0x11 => Some(Error::InvalidIndex),
            0x12 => Some(Error::RFKilled),
            0x13 => Some(Error::AlreadyPaired),
            0x14 => Some(Error::PermissionDenied),
            _ => Some(Error::UnknownError),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::SocketError => f.write_str("SocketError"),
            Error::BindError => f.write_str("BindError"),
            Error::NoResponse => f.write_str("NoResponse"),
            Error::UnknownCommand => f.write_str("UnknownCommand"),
            Error::NotConnected => f.write_str("NotConnected"),
            Error::Failed => f.write_str("Failed"),
            Error::ConnectFailed => f.write_str("ConnectFailed"),
            Error::AuthenticationFailed => f.write_str("AuthenticationFailed"),
            Error::NotPaired => f.write_str("NotPaired"),
            Error::NoResources => f.write_str("NoResources"),
            Error::Timeout => f.write_str("Timeout"),
            Error::AlreadyConnected => f.write_str("AlreadyConnected"),
            Error::Busy => f.write_str("Busy"),
            Error::Rejected => f.write_str("Rejected"),
            Error::NotSupported => f.write_str("NotSupported"),
            Error::InvalidParameters => f.write_str("InvalidParameters"),
            Error::Disconnected => f.write_str("Disconnected"),
            Error::NotPowered => f.write_str("NotPowered"),
            Error::Canceled => f.write_str("Canceled"),
            Error::InvalidIndex => f.write_str("InvalidIndex"),
            Error::RFKilled => f.write_str("RFKilled"),
            Error::AlreadyPaired => f.write_str("AlreadyPaired"),
            Error::PermissionDenied => f.write_str("PermissionDenied"),
            Error::UnknownError => f.write_str("UnknownError"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::SocketError => "Error opening socket",
            Error::BindError => "Error binding to a socket",
            Error::NoResponse => "Response has not been stored",
            Error::UnknownCommand => "Unknown command",
            Error::NotConnected => "Not connected",
            Error::Failed => "Failed",
            Error::ConnectFailed => "Connect failed",
            Error::AuthenticationFailed => "Authentication failed",
            Error::NotPaired => "Not paired",
            Error::NoResources => "No resources",
            Error::Timeout => "Timeout",
            Error::AlreadyConnected => "Already connected",
            Error::Busy => "Busy",
            Error::Rejected => "Rejected",
            Error::NotSupported => "Not supported",
            Error::InvalidParameters => "Invalid parameters",
            Error::Disconnected => "Disconnected",
            Error::NotPowered => "Not powered",
            Error::Canceled => "Canceled",
            Error::InvalidIndex => "Invalid index",
            Error::RFKilled => "RF killed",
            Error::AlreadyPaired => "Already paired",
            Error::PermissionDenied => "Permission denied",
            Error::UnknownError => "Unknown error",
        }
    }
}
