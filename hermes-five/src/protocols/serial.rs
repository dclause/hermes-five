//! This file contains the `SerialProtocol
//! ` code.
//! It allows communication of boards connected via a serial port to HERMES.
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::trace;
use serialport::{DataBits, FlowControl, Parity, StopBits};
use serialport::SerialPort;
use snafu::prelude::*;

use crate::protocols::*;
use crate::protocols::Error::*;

/// The `SerialProtocol` is made to communicate with a remote board using the serial protocol.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct SerialProtocol {
    /// The connection port.
    port: String,
    /// A Read/Write io object.
    #[cfg_attr(feature = "serde", serde(skip))]
    io: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
}

impl SerialProtocol {
    /// Builds a new `SerialProtocol` instance for communication through the given port.
    ///
    /// # Example
    /// ```
    /// let protocol = SerialProtocol::new("/dev/ttyACM0");
    /// let board = Board::default().with_protocol(protocol).open().await;
    /// ```
    pub fn new<'a, P: Into<Cow<'a, str>>>(port: P) -> Self {
        let port_cow = port.into();
        let port = port_cow.as_ref();
        Self {
            port: port.to_string(),
            io: Arc::new(Mutex::new(None)),
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
    /// Open the communication with the registered port.
    fn open(&mut self) -> Result<(), Error> {
        trace!("Open serial protocol on port: {}", self.port);

        let connexion = serialport::new(self.port.clone(), 57_600)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(1000))
            .open()
            .context(SerialPortSnafu)?;
        self.io = Arc::new(Mutex::new(Some(connexion)));

        Ok(())
    }

    /// Gracefully shuts down the serial port communication.
    fn close(&mut self) -> Result<(), Error> {
        trace!("Close serial protocol on port: {}", self.port);
        *self.io.lock().map_err(|_| MutexPoison)? = None;
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.io
            .lock()
            .map_err(|_| MutexPoison)?
            .as_mut()
            .ok_or(NotInitialized)?
            .write(buf)
            .context(IoExceptionSnafu)
    }
}

impl Display for SerialProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let io = self.io.lock().unwrap();
        write!(
            f,
            "firmware={}, version={}, protocol={}, connection={:?}",
            "todo",
            "todo",
            // self.firmware,
            // self.version,
            "SerialProtocol",
            io
        )
    }
}
