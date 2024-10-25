use crate::errors::Error;
use crate::errors::ProtocolError::NotInitialized;
use crate::io::firmata::TransportLayer;
use log::trace;
use parking_lot::Mutex;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;

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
    /// # Arguments
    /// * `port` - The serial port to use for communication.
    ///
    /// # Example
    /// ```
    /// use hermes_five::hardware::Board;
    /// use hermes_five::io::FirmataIO;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let protocol = FirmataIO::new("/dev/ttyACM0");
    ///     let board = Board::from(protocol).open();
    /// }
    /// ```
    pub fn new<P: Into<String>>(port: P) -> Self {
        Self {
            port: port.into(),
            io: Arc::new(Mutex::new(None)),
        }
    }

    /// Retrieves the configured port.
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
        write!(f, "Serial({})", self.port)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl TransportLayer for Serial {
    fn open(&mut self) -> Result<(), Error> {
        let connexion = serialport::new(self.port.clone(), 57_600)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_secs(10))
            .open_native()?;
        trace!("Serial port is now opened: {:?}", connexion);

        // Save the IO (required by handshake).
        self.io = Arc::new(Mutex::new(Some(Box::new(connexion))));

        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        *self.io.lock() = None;
        Ok(())
    }

    fn set_timeout(&mut self, duration: Duration) -> Result<(), Error> {
        self.io.lock().as_mut().unwrap().set_timeout(duration)?;
        Ok(())
    }

    /// Write bytes to the internal connection. For more details see [`std::io::Write::write`].
    ///
    /// # Arguments
    /// * `buf` - The data to write.
    ///
    /// # Returns
    /// * `Ok(())` if all bytes were successfully written.
    /// * `Err(Error)` if there was an issue writing data.
    ///
    /// # Notes
    /// This function blocks until the write operation is complete. Ensure proper error handling in calling code.
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut lock = self.io.lock();
        lock.as_mut().ok_or(NotInitialized)?.write_all(buf)?;
        Ok(())
    }

    /// Reads from the internal connection. For more details see [`std::io::Read::read_exact`].
    ///
    /// # Arguments
    /// * `buf` - The buffer to fill with read data.
    ///
    /// # Returns
    /// * `Ok(())` if the buffer was filled successfully.
    /// * `Err(Error)` if there was an issue reading data.
    ///
    /// # Notes
    /// This function blocks until the buffer is filled or an error occurs. Ensure proper error handling in calling code.
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
        assert!(protocol.port.len() > 0);
    }

    // #[test]
    // fn test_open_serial_protocol() {
    //     let mut protocol = get_test_successful_protocol();
    //     let result = protocol.open();
    //     assert!(result.is_ok());
    //
    //     let mut protocol = get_test_failing_protocol();
    //     let result = protocol.open();
    //     assert!(result.is_err());
    // }

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
        assert_eq!(custom_error.to_string(), "PluginIO error: test error.");

        let serial_error = serialport::Error {
            kind: ErrorKind::Io(std::io::ErrorKind::NotFound),
            description: String::from("IO error"),
        };
        let custom_error: Error = serial_error.into();
        assert_eq!(
            custom_error.to_string(),
            "PluginIO error: Board not found or already in use."
        );

        let serial_error = serialport::Error {
            kind: ErrorKind::Io(std::io::ErrorKind::Other),
            description: String::from("IO error"),
        };
        let custom_error: Error = serial_error.into();
        assert_eq!(custom_error.to_string(), "PluginIO error: IO error.");
    }

    #[test]
    fn test_display_serial_protocol() {
        let protocol = Serial::new("/dev/ttyACM0");
        assert_eq!(format!("{}", protocol), "Serial(/dev/ttyACM0)");
    }
}
