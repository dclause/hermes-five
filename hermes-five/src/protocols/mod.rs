use std::fmt::{Debug, Display};

use dyn_clone::DynClone;
use snafu::prelude::*;

use crate::protocols::firmata::*;

mod firmata;
mod pins;
pub mod serial;

/// Received Protocol message.
#[derive(Clone, Debug)]
pub enum Message {
    ProtocolVersion,
    Analog,
    Digital,
    EmptyResponse,
    AnalogMappingResponse,
    CapabilityResponse,
    PinStateResponse,
    ReportFirmware,
    I2CReply,
}

/// Firmata error type.
#[derive(Debug, Snafu)]
pub enum Error {
    // /// Unknown SysEx code: {code}.
    // UnknownSysEx { code: u8 },
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
    // /// UTF8 error: {source}.
    // Utf8Error { source: std::str::Utf8Error },
    // /// Message was too short.
    // MessageTooShort,
    // /// Unknown pin {pin} (max {len}).
    // PinOutOfBounds { pin: u8, len: usize },
    /// An error occurred: {info}
    Unknown { info: String },
    /// Serial port error: {source}
    SerialPort { source: serialport::Error },
}

/// Defines the trait all protocols must implements.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Protocol: DynClone + Send + Sync + Display {
    /// Open the communication using the underlying protocol.
    fn open(&mut self) -> Result<(), Error>;
    /// Gracefully shuts down the communication.
    fn close(&mut self) -> Result<(), Error>;
    /// Write data in the protocol.
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;

    /// Starts a conversation with the board: validate the firmware version and...
    fn handshake(&mut self) -> Result<(), Error> {
        self.query_firmware()?;
        // self.read_and_decode()?;
        // self.query_capabilities()?;
        // self.read_and_decode()?;
        // self.query_analog_mapping()?;
        // self.read_and_decode()?;
        // self.report_digital(0, 1)?;
        // self.report_digital(1, 1)?;
        Ok(())
    }

    // ########################################
    // Protocol related functions

    /// Query the board for current firmware and protocol information.
    fn query_firmware(&mut self) -> Result<(), Error> {
        self.write(&[START_SYSEX, REPORT_FIRMWARE, END_SYSEX])?;
        Ok(())
    }

    /// Query the board for all available capabilities.
    fn query_capabilities(&mut self) -> Result<(), Error> {
        self.write(&[START_SYSEX, CAPABILITY_QUERY, END_SYSEX])?;
        Ok(())
    }

    /// Query the board for available analog pins.
    fn query_analog_mapping(&mut self) -> Result<(), Error> {
        self.write(&[START_SYSEX, ANALOG_MAPPING_QUERY, END_SYSEX])?;
        Ok(())
    }

    // ########################################
    // Read/Write on pins
    //
    // /// Write `level` to the analog `pin`.
    // fn analog_write(&mut self, pin: i32, level: i32) -> Result<(), Error>;
    // /// Write `level` to the digital `pin`.
    // fn digital_write(&mut self, pin: i32, level: i32) -> Result<(), Error>;
    // /// Set the analog reporting `state` of the specified `pin`.
    // fn report_analog(&mut self, pin: i32, state: i32) -> Result<(), Error>;
    // /// Set the digital reporting `state` of the specified `pin`.
    // fn report_digital(&mut self, pin: i32, state: i32) -> Result<(), Error>;
    // /// Set the `mode` of the specified `pin`.
    // fn set_pin_mode(&mut self, pin: i32, mode: u8) -> Result<(), Error>;
    //
    // ########################################
    // I2C
    //
    // /// Configure the `delay` in microseconds for I2C devices that require a delay between when the
    // /// register is written to and the data in that register can be read.
    // fn i2c_config(&mut self, delay: i32) -> Result<(), Error>;
    // /// Get the raw I2C replies that have been read from the board.
    // fn i2c_data(&mut self) -> &mut Vec<I2CReply>;
    // /// Read `size` bytes from I2C device at the specified `address`.
    // fn i2c_read(&mut self, address: i32, size: i32) -> Result<(), Error>;
    // /// Write `data` to the I2C device at the specified `address`.
    // fn i2c_write(&mut self, address: i32, data: &[u8]) -> Result<(), Error>;

    // ########################################
    // Firmata

    // /// Read from the Firmata device, parse one Firmata message and return its type.
    // fn read_and_decode(&mut self) -> Result<Message> {
    //     let mut buf = vec![0; 3];
    //     self.read_exact(&mut buf)?;
    //     match buf[0] {
    //         REPORT_VERSION => {
    //             todo!();
    //             Ok(Message::ProtocolVersion)
    //         }
    //         ANALOG_MESSAGE..=ANALOG_MESSAGE_BOUND => {
    //             if buf.len() < 3 {
    //                 bail!(Error::MessageTooShort);
    //             }
    //             let pin = ((buf[0] as i32) & 0x0F) + 14;
    //             let value = (buf[1] as i32) | ((buf[2] as i32) << 7);
    //             if self.get_pins().len() as i32 > pin {
    //                 self.set_pin_value(pin as usize, value);
    //             }
    //             Ok(Message::Analog)
    //         }
    //         DIGITAL_MESSAGE..=DIGITAL_MESSAGE_BOUND => {
    //             if buf.len() < 3 {
    //                 return Err(Error::MessageTooShort);
    //             }
    //             let port = (buf[0] as i32) & 0x0F;
    //             let value = (buf[1] as i32) | ((buf[2] as i32) << 7);
    //
    //             for i in 0..8 {
    //                 let pin = (8 * port) + i;
    //                 let mode: u8 = self.pins[pin as usize].mode;
    //                 if self.pins.len() as i32 > pin && mode == PIN_MODE_INPUT {
    //                     self.pins[pin as usize].value = (value >> (i & 0x07)) & 0x01;
    //                 }
    //             }
    //             Ok(Message::Digital)
    //         }
    //         START_SYSEX => {
    //             loop {
    //                 // Read until END_SYSEX.
    //                 let mut byte = [0];
    //                 self.connection
    //                     .read_exact(&mut byte)
    //                     .with_context(|_| StdIoSnafu)?;
    //                 buf.push(byte[0]);
    //                 if byte[0] == END_SYSEX {
    //                     break;
    //                 }
    //             }
    //             match buf[1] {
    //                 END_SYSEX => Ok(Message::EmptyResponse),
    //                 ANALOG_MAPPING_RESPONSE => {
    //                     let mut i = 2;
    //                     // Also break before pins indexing is out of bounds.
    //                     let upper = (buf.len() - 1).min(self.pins.len() + 2);
    //                     while i < upper {
    //                         if buf[i] != 127u8 {
    //                             let pin = &mut self.pins[i - 2];
    //                             pin.mode = PIN_MODE_ANALOG;
    //                             pin.modes = vec![PIN_MODE_ANALOG];
    //                             pin.resolution = DEFAULT_ANALOG_RESOLUTION;
    //                         }
    //                         i += 1;
    //                     }
    //                     Ok(Message::AnalogMappingResponse)
    //                 }
    //                 CAPABILITY_RESPONSE => {
    //                     let mut i = 2;
    //                     self.pins = vec![];
    //                     self.pins.push(Pin::default()); // 0 is unused.
    //                     let mut modes = vec![];
    //                     let mut resolution = None;
    //                     while i < buf.len() - 1 {
    //                         // Completed a pin, push and continue.
    //                         if buf[i] == 127u8 {
    //                             self.pins.push(Pin {
    //                                 mode: *modes.first().expect("pin mode"),
    //                                 modes: modes.drain(..).collect(),
    //                                 resolution: resolution.take().expect("pin resolution"),
    //                                 value: 0,
    //                             });
    //
    //                             i += 1;
    //                         } else {
    //                             modes.push(buf[i]);
    //                             if resolution.is_none() {
    //                                 // Only keep the first.
    //                                 resolution.replace(buf[i + 1]);
    //                             }
    //                             i += 2;
    //                         }
    //                     }
    //                     Ok(Message::CapabilityResponse)
    //                 }
    //                 REPORT_FIRMWARE => {
    //                     let major = buf.get(2).with_context(|| MessageTooShortSnafu)?;
    //                     let minor = buf.get(3).with_context(|| MessageTooShortSnafu)?;
    //                     self.firmware_version = format!("{:o}.{:o}", major, minor);
    //                     if 4 < buf.len() - 1 {
    //                         self.firmware_name = std::str::from_utf8(&buf[4..buf.len() - 1])
    //                             .with_context(|_| Utf8Snafu)?
    //                             .to_string();
    //                     }
    //                     Ok(Message::ReportFirmware)
    //                 }
    //                 I2C_REPLY => {
    //                     let len = buf.len();
    //                     if len < 8 {
    //                         return Err(Error::MessageTooShort);
    //                     }
    //                     let mut reply = I2CReply {
    //                         address: (buf[2] as i32) | ((buf[3] as i32) << 7),
    //                         register: (buf[4] as i32) | ((buf[5] as i32) << 7),
    //                         data: vec![buf[6] | buf[7] << 7],
    //                     };
    //                     let mut i = 8;
    //
    //                     while i < len - 1 {
    //                         if buf[i] == 0xF7 {
    //                             break;
    //                         }
    //                         if i + 2 > len {
    //                             break;
    //                         }
    //                         reply.data.push(buf[i] | buf[i + 1] << 7);
    //                         i += 2;
    //                     }
    //                     self.i2c_data.push(reply);
    //                     Ok(Message::I2CReply)
    //                 }
    //                 PIN_STATE_RESPONSE => {
    //                     let pin = buf[2];
    //                     if buf[3] == END_SYSEX {
    //                         return Ok(Message::PinStateResponse);
    //                     }
    //                     let pin = &mut self.pins[pin as usize];
    //                     pin.modes = vec![buf[3]];
    //                     // TODO: Extended values.
    //                     pin.value = buf[4] as i32;
    //
    //                     Ok(Message::PinStateResponse)
    //                 }
    //                 _ => Err(Error::UnknownSysEx { code: buf[1] }),
    //             }
    //         }
    //         _ => Err(Error::BadByte { byte: buf[0] }),
    //     }
    // }
}

// Makes a Box<dyn Protocol> clone (used for Board cloning).
dyn_clone::clone_trait_object!(Protocol);
