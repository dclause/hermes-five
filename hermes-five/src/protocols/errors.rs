use snafu::Snafu;

use crate::protocols::PinModeId;

/// Firmata error type.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// Communication error: Unknown SysEx code: {code}.
    UnknownSysEx { code: u8 },
    /// Received a bad byte: {byte}.
    BadByte { byte: u8 },
    /// Protocol error: not initialized.
    NotInitialized,
    /// Protocol error: device currently uses {version}. This application requires 3.5.6 or later.
    ProtocolVersion { version: String },
    /// I/O error: {source}.
    IoException { source: std::io::Error },
    /// Mutex error: The Mutex holding the port was poisoned
    MutexPoison,
    /// UTF8 error: {source}.
    Utf8Error { source: std::str::Utf8Error },
    /// Data error: Not enough bytes received, message was too short.
    MessageTooShort,
    /// Unknown error: {info}
    Unknown { info: String },
    /// Serial port error: {source}
    SerialPort { source: serialport::Error },
    /// {info}
    Custom { info: String },

    // ##### PIN RELATED #####
    /// Unknown pin {pin}.
    UnknownPin { pin: u16 },
    /// Incompatible pin {pin}.
    IncompatiblePin { pin: u16 },
    /// The value ({value}) is not compatible with the current pin mode.
    IncompatibleValue { value: u16 },
    /// Unknown mode {mode}.
    UnknownMode { mode: PinModeId },
    /// The mode ({mode}) is not compatible with pin {pin}: "{operation}".
    IncompatibleMode {
        mode: PinModeId,
        pin: u16,
        operation: String,
    },
}
