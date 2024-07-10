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
use crate::protocols::errors::{IoExceptionSnafu, SerialPortSnafu};
use crate::protocols::pins::Pin;

/// The `SerialProtocol` is made to communicate with a remote board using the serial protocol.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct SerialProtocol {
    /// The connection port.
    port: String,
    /// A Read/Write io object.
    #[cfg_attr(feature = "serde", serde(skip))]
    io: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
    pins: Vec<Pin>,
    i2c_data: Vec<I2CReply>,
    protocol_version: String,
    firmware_name: String,
    firmware_version: String,
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
            pins: vec![],
            i2c_data: vec![],
            protocol_version: String::default(),
            firmware_name: String::default(),
            firmware_version: String::default(),
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

    fn pins(&mut self) -> &mut Vec<Pin> {
        &mut self.pins
    }
    fn with_pins(&mut self, pins: Vec<Pin>) {
        self.pins = pins;
    }
    fn protocol_version(&mut self) -> &String {
        &self.protocol_version
    }
    fn with_protocol_version(&mut self, protocol_version: String) {
        self.protocol_version = protocol_version.into();
    }
    fn firmware_name(&mut self) -> &String {
        &self.firmware_name
    }
    fn with_firmware_name(&mut self, firmware_name: String) {
        self.firmware_name = firmware_name.into();
    }
    fn firmware_version(&mut self) -> &String {
        &self.firmware_version
    }
    fn with_firmware_version(&mut self, firmware_version: String) {
        self.firmware_version = firmware_version.into();
    }
    fn i2c_data(&mut self) -> &mut Vec<I2CReply> {
        &mut self.i2c_data
    }
    fn with_i2c_data(&mut self, i2c_data: Vec<I2CReply>) {
        self.i2c_data = i2c_data;
    }

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

    /// Write to  the internal connection. For more details see [`std::io::Write::write`].
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut lock = self.io.lock().map_err(|_| MutexPoison)?;
        let bytes_written = lock
            .as_mut()
            .ok_or(NotInitialized)?
            .write(buf)
            .context(IoExceptionSnafu)?;

        // Check if all bytes were successfully written
        if bytes_written == buf.len() {
            Ok(())
        } else {
            Err(MessageTooShort)
        }
    }

    /// Read from the internal connection. For more details see [`std::io::Read::read_exact`].
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let mut lock = self.io.lock().map_err(|_| MutexPoison)?;
        lock.as_mut()
            .ok_or(NotInitialized)?
            .read_exact(buf)
            .context(IoExceptionSnafu)
    }
}

// @todo Make [`Self::io`] generic (Read + Write + Debug) so we can actually make this 'dyn Protocol' generic
impl Display for SerialProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let io = self.io.lock().unwrap();
        write!(
            f,
            "firmware={}, version={}, protocol={}, connection={:?}",
            self.firmware_name, self.firmware_version, "SerialProtocol", io
        )
    }
}
