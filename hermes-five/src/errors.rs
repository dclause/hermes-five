use std::str::Utf8Error;

use log::error;
use snafu::Snafu;

pub use crate::errors::Error::*;
use crate::errors::ProtocolError::IoException;
use crate::protocols::PinModeId;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// Runtime error: Are you sure your code runs inside #[hermes_five::runtime]?
    RuntimeError,
    /// Protocol error: {source}.
    ProtocolError { source: ProtocolError },
    /// Hardware error: {source}.
    HardwareError { source: HardwareError },
    /// Unknown error: {info}.
    Unknown { info: String },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        error!("std::io error {:?}", error);
        let info = match error.kind() {
            std::io::ErrorKind::NotFound => String::from("Board not found or already in use"),
            std::io::ErrorKind::PermissionDenied => String::from("Board connection lost"),
            _ => error.to_string(),
        };
        Self::ProtocolError {
            source: IoException { info: info },
        }
    }
}

impl From<ProtocolError> for Error {
    fn from(value: ProtocolError) -> Self {
        Self::ProtocolError { source: value }
    }
}

impl From<HardwareError> for Error {
    fn from(value: HardwareError) -> Self {
        Self::HardwareError { source: value }
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::Unknown {
            info: value.to_string(),
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ProtocolError {
    /// {info}
    IoException { info: String },
    /// Connection has not been initialized
    NotInitialized,
    /// Not enough bytes received - '{operation}' expected {expected} bytes, {received} received
    MessageTooShort {
        operation: &'static str,
        expected: usize,
        received: usize,
    },
    /// Unexpected data received
    UnexpectedData,
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum HardwareError {
    /// Pin ({pin}) not compatible with mode ({mode}) - {context}
    IncompatibleMode {
        pin: u16,
        mode: PinModeId,
        context: &'static str,
    },
    /// Unknown pin {pin}
    UnknownPin { pin: u16 },
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::errors::HardwareError::{IncompatibleMode, UnknownPin};

    use super::*;

    #[test]
    fn test_error_display() {
        let runtime_error = RuntimeError;
        assert_eq!(
            format!("{}", runtime_error),
            "Runtime error: Are you sure your code runs inside #[hermes_five::runtime]?"
        );

        let protocol_error = Error::from(IoException {
            info: "I/O error message".to_string(),
        });
        assert_eq!(
            format!("{}", protocol_error),
            "Protocol error: I/O error message."
        );

        let hardware_error = Error::from(IncompatibleMode {
            pin: 1,
            mode: PinModeId::SERVO,
            context: "test context",
        });
        assert_eq!(
            format!("{}", hardware_error),
            "Hardware error: Pin (1) not compatible with mode (SERVO) - test context."
        );

        let unknown_error = Unknown {
            info: "Some unknown error".to_string(),
        };
        assert_eq!(
            format!("{}", unknown_error),
            "Unknown error: Some unknown error."
        );
    }

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error: Error = io_error.into();
        assert_eq!(
            format!("{}", error),
            "Protocol error: Board not found or already in use."
        );
    }

    #[test]
    fn test_from_protocol_error() {
        let protocol_error = ProtocolError::NotInitialized;
        let error: Error = protocol_error.into();
        assert_eq!(
            format!("{}", error),
            "Protocol error: Connection has not been initialized."
        );
    }

    #[test]
    fn test_from_hardware_error() {
        let hardware_error = UnknownPin { pin: 42 };
        let error: Error = hardware_error.into();
        assert_eq!(format!("{}", error), "Hardware error: Unknown pin 42.");
    }

    #[test]
    fn test_from_utf8_error() {
        #[allow(invalid_from_utf8)]
        let utf8_error = std::str::from_utf8(&[0x80]).err().unwrap(); // Invalid UTF-8 sequence
        let error: Error = utf8_error.into();
        assert_eq!(
            format!("{}", error),
            "Unknown error: invalid utf-8 sequence of 1 bytes from index 0."
        )
    }
}
