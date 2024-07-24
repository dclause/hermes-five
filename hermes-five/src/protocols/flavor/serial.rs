//! This file contains the `SerialProtocol
//! ` code.
//! It allows communication of boards connected via a serial port to HERMES.
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use log::trace;
use parking_lot::{Mutex, RwLock};
use serialport::{DataBits, ErrorKind, FlowControl, Parity, StopBits};
use serialport::SerialPort;

use crate::errors::Error;
use crate::errors::ProtocolError::{IoException, MessageTooShort, NotInitialized};
use crate::protocols::{Hardware, Protocol};
use crate::protocols::protocol::ProtocolHardware;

/// The `SerialProtocol` is made to communicate with a remote board using the serial protocol.
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
    hardware: ProtocolHardware,
}

impl SerialProtocol {
    /// Builds a new `SerialProtocol` instance for communication through the given port.
    ///
    /// # Example
    /// ```
    /// let protocol = SerialProtocol::new("/dev/ttyACM0");
    /// let board = Board::default().with_protocol(protocol).open();
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
    /// Creates a new default SerialProtocol.
    /// The first available port will be used, None otherwise, which will probably lead to an error
    /// during the open phase.
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
    fn hardware(&self) -> &ProtocolHardware {
        &self.hardware
    }

    fn get_protocol_details(&self) -> String {
        format!("via port {}", self.port)
    }

    // ########################################
    // Protocol related functions

    /// Open the communication with the registered port.
    fn open(&mut self) -> Result<(), Error> {
        trace!("Open serial protocol on port: {}", self.port);

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
    fn close(&mut self) -> Result<(), Error> {
        trace!("Close serial protocol on port: {}", self.port);
        *self.io.lock() = None;
        Ok(())
    }

    /// Write to  the internal connection. For more details see [`std::io::Write::write`].
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

    /// Read from the internal connection. For more details see [`std::io::Read::read_exact`].
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
