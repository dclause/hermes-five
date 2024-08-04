//! This file contains the `SerialProtocol` code.
//!
//! It allows communication of boards connected via a serial port.
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use serialport::{DataBits, ErrorKind, FlowControl, Parity, StopBits};
use serialport::SerialPort;

use crate::errors::Error;
use crate::errors::ProtocolError::{IoException, MessageTooShort, NotInitialized};
use crate::protocols::{Hardware, Protocol};

/// The `SerialProtocol` is made to communicate with a remote board using the serial protocol.
///
/// # Fields
/// * `port`: The connection port.
/// * `io`: A Read/Write io object.
/// * `hardware`: The base-protocol attributes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct SerialProtocol {
    /// The connection port.
    port: String,
    /// A Read/Write io object.
    #[cfg_attr(feature = "serde", serde(skip))]
    io: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
    /// The base-protocol attributes.
    #[cfg_attr(feature = "serde", serde(skip))]
    hardware: Arc<RwLock<Hardware>>,
}

impl SerialProtocol {
    /// Constructs a new `SerialProtocol` instance for communication through the specified port.
    ///
    /// # Arguments
    /// * `port` - The serial port to use for communication.
    ///
    /// # Example
    /// ```
    /// use hermes_five::Board;
    /// use hermes_five::protocols::SerialProtocol;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let protocol = SerialProtocol::new("/dev/ttyACM0");
    ///     let board = Board::default().with_protocol(protocol).open();
    /// }
    /// ```
    pub fn new<P: Into<String>>(port: P) -> Self {
        Self {
            port: port.into(),
            io: Arc::new(Mutex::new(None)),
            hardware: Arc::new(RwLock::new(Hardware::default())),
        }
    }
}

impl Default for SerialProtocol {
    /// Creates a new `SerialProtocol` with the first available port or an empty string if no ports are available.
    ///
    /// # Notes
    /// The first available port will be used, None otherwise, which will probably lead to an error
    /// during the open phase.
    #[cfg(not(tarpaulin_include))]
    fn default() -> Self {
        let ports = serialport::available_ports().unwrap_or_else(|_| vec![]);
        match ports.get(0) {
            Some(port) => Self::new(&port.port_name),
            None => Self::new(""),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Protocol for SerialProtocol {
    // ########################################
    // Inner data related functions

    /// Retrieve the internal hardware.
    fn get_hardware(&self) -> &Arc<RwLock<Hardware>> {
        &self.hardware
    }

    fn get_protocol_details(&self) -> String {
        format!("via port {}", self.port)
    }

    // ########################################
    // Protocol related functions

    /// Opens communication with the specified port.
    ///
    /// # Returns
    /// * `Ok(())` if successful.
    /// * `Err(Error)` if there is an issue opening the port.
    ///
    /// # Notes
    /// - The method is sync and will block until the serial port is open.
    /// - This function initializes the `io` object. Ensure the port is valid before calling this method.
    #[cfg(not(tarpaulin_include))]
    fn open(&mut self) -> Result<(), Error> {
        let connexion = serialport::new(self.port.clone(), 57_600)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(1000))
            .open()?;
        self.io = Arc::new(Mutex::new(Some(connexion)));

        Ok(())
    }

    /// Gracefully shuts down the serial port communication.
    ///
    /// # Returns
    /// * `Ok(())` if successful.
    /// * `Err(Error)` if there is an issue closing the port.
    fn close(&mut self) -> Result<(), Error> {
        *self.io.lock() = None;
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
        let bytes_written = lock.as_mut().ok_or(NotInitialized)?.write(buf)?;

        // Check if all bytes were successfully written
        match bytes_written == buf.len() {
            true => Ok(()),
            false => Err(MessageTooShort {
                operation: "write",
                expected: buf.len(),
                received: bytes_written,
            }),
        }?;

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
        let info = match value.kind {
            ErrorKind::Io(kind) => match kind {
                std::io::ErrorKind::Other => {
                    String::from("Port connexion not found: check if board is connected")
                }
                _ => value.to_string(),
            },
            _ => value.to_string(),
        };
        Error::ProtocolError {
            source: IoException { info },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mocks::io::SerialPortMock;

    use super::*;

    fn get_test_successful_protocol() -> SerialProtocol {
        let protocol = SerialProtocol::new("/dev/ttyACM0");
        *protocol.io.lock() = Some(Box::new(SerialPortMock::default()));
        protocol
    }

    fn get_test_failing_protocol() -> SerialProtocol {
        let protocol = SerialProtocol::new("/dev/ttyACM0");
        *protocol.io.lock() = Some(Box::new(SerialPortMock::new(ErrorKind::InvalidInput)));
        protocol
    }

    #[test]
    fn test_new_serial_protocol() {
        let protocol = SerialProtocol::new("/dev/ttyACM0");
        assert_eq!(protocol.port, "/dev/ttyACM0");
        assert!(protocol.io.lock().is_none());
        assert_eq!(protocol.get_hardware().read().firmware_name, "");
        assert_eq!(protocol.get_protocol_details(), "via port /dev/ttyACM0");
    }

    #[test]
    fn test_default_serial_protocol() {
        let protocol = SerialProtocol::default();
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
        assert!(result.is_err());

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
            kind: serialport::ErrorKind::Unknown,
            description: String::from("test error"),
        };
        let custom_error: Error = serial_error.into();
        assert_eq!(custom_error.to_string(), "Protocol error: test error.");

        let serial_error = serialport::Error {
            kind: serialport::ErrorKind::Io(std::io::ErrorKind::NotFound),
            description: String::from("IO error"),
        };
        let custom_error: Error = serial_error.into();
        assert_eq!(custom_error.to_string(), "Protocol error: IO error.");

        let serial_error = serialport::Error {
            kind: serialport::ErrorKind::Io(std::io::ErrorKind::Other),
            description: String::from("IO error"),
        };
        let custom_error: Error = serial_error.into();
        assert_eq!(
            custom_error.to_string(),
            "Protocol error: Port connexion not found: check if board is connected."
        );
    }
}
