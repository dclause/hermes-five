//! This file contains the `SerialProtocol
//! ` code.
//! It allows communication of boards connected via a serial port to HERMES.
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{bail, format_err, Result};
use log::trace;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

use crate::protocols::Protocol;

/// The `SerialProtocol` is made to communicate with a remote board using the serial protocol.
#[derive(Clone)]
#[cfg_attr(feature = "storage", derive(serde::Serialize, serde::Deserialize))]
pub struct SerialProtocol {
    // The connection port
    port: String,
    #[cfg_attr(feature = "storage", serde(skip))]
    connexion: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
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
            connexion: Arc::new(Mutex::new(None)),
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

#[cfg_attr(feature = "storage", typetag::serde)]
impl Protocol for SerialProtocol {
    /// Open the communication with the registered port.
    fn open(&mut self) -> Result<()> {
        trace!("Open serial protocol on port: {}", self.port);

        let connexion = match serialport::new(self.port.clone(), 57_600)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(1000))
            .open()
        {
            Ok(connexion) => connexion,
            Err(err) => bail!("Error opening port ({}): {}", self.port, err.description),
        };
        self.connexion = Arc::new(Mutex::new(Some(connexion)));

        Ok(())
    }

    /// Gracefully shuts down the serial port communication.
    fn close(&mut self) -> Result<()> {
        trace!("Close serial protocol on port: {}", self.port);
        *self.connexion.lock().map_err(|err| {
            format_err!("Error closing port ({}): {}", self.port, err.to_string())
        })? = None;
        Ok(())
    }
}

impl Debug for SerialProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("").field(&self.port).finish()
    }
}
