use crate::errors::Error;
use crate::errors::ProtocolError::NotInitialized;
use crate::io::IoTransport;
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
    /// use hermes_five::io::RemoteIo;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let protocol = RemoteIo::new("/dev/ttyACM0");
    ///     let board = Board::new(protocol).open();
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

impl Default for Serial {
    /// Creates a new serial transport connection with the first available port or an empty string if no ports are available.
    ///
    /// # Notes
    /// The first available port will be used, None otherwise, which will probably lead to an error
    /// during the open phase.
    #[cfg(not(tarpaulin_include))]
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

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoTransport for Serial {
    fn open(&mut self) -> Result<(), Error> {
        let connexion = serialport::new(self.port.clone(), 57_600)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_secs(10))
            .open_native()?;

        // Save the IO (required by handshake).
        self.io = Arc::new(Mutex::new(Some(Box::new(connexion))));

        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        *self.io.lock() = None;
        Ok(())
    }

    fn set_timeout(&mut self, duration: Duration) -> Result<(), Error> {
        self.io
            .lock()
            .as_mut()
            .ok_or(NotInitialized)?
            .set_timeout(duration)?;
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut lock = self.io.lock();
        lock.as_mut().ok_or(NotInitialized)?.write_all(buf)?;
        Ok(())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
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
    use crate::mocks::serial_port::SerialPortMock;
    use serialport::ErrorKind;

    fn get_test_successful_protocol() -> Serial {
        let protocol = Serial::new("/dev/ttyACM0");
        *protocol.io.lock() = Some(Box::new(SerialPortMock::default()));
        protocol
    }

    fn get_test_failing_protocol() -> Serial {
        let protocol = Serial::new("/dev/ttyACM0");
        *protocol.io.lock() = Some(Box::new(SerialPortMock::new(ErrorKind::InvalidInput)));
        protocol
    }

    #[test]
    fn test_new_serial_protocol() {
        let protocol = Serial::new("/dev/ttyACM0");
        assert_eq!(protocol.port, "/dev/ttyACM0");
        assert!(protocol.io.lock().is_none());
    }

    #[test]
    fn test_default_serial_protocol() {
        let protocol = Serial::default();
        assert!(!protocol.port.is_empty());
    }

    #[test]
    fn test_open_serial_protocol() {
        // let mut protocol = get_test_successful_protocol();
        // let result = protocol.open();
        // assert!(result.is_ok());

        let mut protocol = get_test_failing_protocol();
        let result = protocol.open();
        assert!(result.is_err());
    }

    #[test]
    fn test_close_serial_protocol() {
        let mut protocol = get_test_successful_protocol();
        let result = protocol.close();
        assert!(result.is_ok());
        assert!(protocol.io.lock().is_none());
    }

    #[test]
    fn test_write_data_success() {
        let mut protocol = get_test_successful_protocol();
        let result = protocol.write(&[1, 2, 3]);
        assert!(result.is_ok());
        let result = protocol.write(&[]);
        assert!(result.is_ok());

        let mut protocol = get_test_failing_protocol();
        let result = protocol.write(&[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_exact_success() {
        let mut protocol = get_test_successful_protocol();
        let mut buf = [0; 3];
        let result = protocol.read_exact(&mut buf);
        assert!(result.is_ok());

        let mut protocol = get_test_failing_protocol();
        let mut buf = [0; 3];
        let result = protocol.read_exact(&mut buf);
        assert!(result.is_err());
    }

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
    fn test_set_timeout() {
        let mut protocol = Serial::new("/dev/ttyACM0");
        assert!(protocol.set_timeout(Duration::from_secs(1)).is_err());
        let mut protocol = get_test_successful_protocol();
        assert!(protocol.set_timeout(Duration::from_secs(1)).is_ok());
    }

    #[test]
    fn test_display_serial_protocol() {
        let protocol = Serial::new("/dev/ttyACM0");
        assert_eq!(format!("{}", protocol), "Serial(/dev/ttyACM0)");
    }
}
