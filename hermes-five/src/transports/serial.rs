use crate::errors::Error;
use crate::errors::ProtocolError::NotInitialized;
use crate::transports::IoTransport;
use parking_lot::Mutex;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;

/// Represents an [`IoTransport`] layer based on a serial connection.
///
/// Uses [serialport](https://crates.io/crates/serialport) crate.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Serial {
    /// The connection port.
    port: String,
    /// A Read/Write io object.
    #[cfg_attr(feature = "serde", serde(skip))]
    io: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
}

impl Serial {
    /// Constructs a new `Serial` transport layer instance for communication through the specified port.
    ///
    /// # Example
    /// ```
    /// use hermes_five::hardware::Board;
    /// use hermes_five::protocols::RemoteIo;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let protocol = RemoteIo::new("/dev/ttyACM0");
    ///     let board = Board::new(protocol).connect().unwrap();
    /// }
    /// ```
    pub fn new<P: Into<String>>(port: P) -> Self {
        Self {
            port: port.into(),
            io: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns  the configured port.
    pub fn get_port(&self) -> String {
        self.port.clone()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Default for Serial {
    /// Creates a new serial transport connection with the first available port or an empty string if no ports are available.
    ///
    /// # Notes
    /// The first available port will be used, None otherwise, which will probably lead to an error
    /// during the open phase.
    fn default() -> Self {
        let ports = serialport::available_ports().unwrap_or_else(|_| vec![]);
        match ports.first() {
            Some(port) => Self::new(&port.port_name),
            None => Self::new(""),
        }
    }
}

impl Display for Serial {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Serial({}{})",
            self.port,
            if self.io.lock().is_some() { " [*]" } else { "" }
        )
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(feature = "serde", typetag::serde)]
impl IoTransport for Serial {
    fn open(&self) -> Result<(), Error> {
        let connection = serialport::new(self.port.clone(), 57_600)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_secs(10))
            .open_native()?;

        // Save the IO (required by handshake).
        *self.io.lock() = Some(Box::new(connection));

        Ok(())
    }

    fn close(&self) -> Result<(), Error> {
        *self.io.lock() = None;
        Ok(())
    }

    fn write(&self, buf: &[u8]) -> Result<(), Error> {
        let mut lock = self.io.lock();
        lock.as_mut().ok_or(NotInitialized)?.write_all(buf)?;
        Ok(())
    }

    fn read_exact(&self, buf: &mut [u8]) -> Result<(), Error> {
        let mut lock = self.io.lock();
        lock.as_mut().ok_or(NotInitialized)?.read_exact(buf)?;
        Ok(())
    }
}

impl From<serialport::Error> for Error {
    fn from(value: serialport::Error) -> Self {
        std::io::Error::from(value).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serialport::ErrorKind;

    #[test]
    fn test_from_serial_error() {
        let serial_error = serialport::Error {
            kind: ErrorKind::Unknown,
            description: String::from("test error"),
        };
        let custom_error: Error = serial_error.into();
        assert_eq!(custom_error.to_string(), "Protocol error: test error.");

        let serial_error = serialport::Error {
            kind: ErrorKind::Io(std::io::ErrorKind::NotFound),
            description: String::from("IO error"),
        };
        let custom_error: Error = serial_error.into();
        assert_eq!(
            custom_error.to_string(),
            "Protocol error: Board not found or already in use."
        );

        let serial_error = serialport::Error {
            kind: ErrorKind::Io(std::io::ErrorKind::Other),
            description: String::from("IO error"),
        };
        let custom_error: Error = serial_error.into();
        assert_eq!(custom_error.to_string(), "Protocol error: IO error.");
    }

    #[test]
    fn test_new_serial_protocol() {
        let protocol = Serial::new("/dev/ttyACM0");
        assert_eq!(protocol.get_port(), "/dev/ttyACM0".to_string());
    }

    #[test]
    fn test_display_serial_protocol() {
        let protocol = Serial::new("/dev/ttyACM0");
        assert_eq!(format!("{}", protocol), "Serial(/dev/ttyACM0)");
    }
}
