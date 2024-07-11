use std::any::type_name;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;
use snafu::ResultExt;

use crate::protocols::constants::*;
use crate::protocols::Error::{BadByte, MessageTooShort, UnknownSysEx};
pub use crate::protocols::errors::Error;
use crate::protocols::errors::Utf8Snafu;
pub use crate::protocols::i2c_reply::I2CReply;
pub use crate::protocols::pins::Pin;
use crate::protocols::protocol::ProtocolHardware;
pub use crate::protocols::serial::SerialProtocol;

pub mod constants;
mod errors;
mod i2c_reply;
mod pins;
mod protocol;
pub mod serial;

// Makes a Box<dyn Protocol> clone (used for Board cloning).
dyn_clone::clone_trait_object!(Protocol);

/// Defines the trait all protocols must implements.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Protocol: DynClone + Send + Sync + Debug {
    // ########################################
    // Inner data related functions

    /// Retrieve the internal hardware.
    fn hardware(&self) -> &ProtocolHardware;
    fn hardware_mut(&mut self) -> &mut ProtocolHardware;

    // ########################################
    // Functions specifically bound to the protocol.

    /// Open the communication using the underlying protocol.
    fn open(&mut self) -> Result<(), Error>;
    /// Gracefully shuts down the communication.
    fn close(&mut self) -> Result<(), Error>;
    /// Write to  the internal connection. For more details see [`std::io::Write::write`].
    fn write(&mut self, buf: &[u8]) -> Result<(), Error>;
    /// Read from the internal connection. For more details see [`std::io::Read::read_exact`].
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error>;

    // ########################################
    // Protocol related functions

    /// Returns the protocol name (used for Display only)
    fn get_protocol_name(&self) -> &'static str {
        type_name::<Self>().split("::").last().unwrap()
    }

    /// Returns the protocol internal details (used for Display only)
    fn get_protocol_details(&self) -> String {
        String::from("()")
    }

    /// Starts a conversation with the board: validate the firmware version and...
    fn handshake(&mut self) -> Result<(), Error> {
        self.query_firmware()?;
        self.read_and_decode()?;
        self.query_capabilities()?;
        self.read_and_decode()?;
        self.query_analog_mapping()?;
        self.read_and_decode()?;
        self.report_digital(0, 1)?;
        self.report_digital(1, 1)?;
        Ok(())
    }

    /// Query the board for current firmware and protocol information.
    fn query_firmware(&mut self) -> Result<(), Error> {
        self.write(&[START_SYSEX, REPORT_FIRMWARE, END_SYSEX])
    }

    /// Query the board for all available capabilities.
    fn query_capabilities(&mut self) -> Result<(), Error> {
        self.write(&[START_SYSEX, CAPABILITY_QUERY, END_SYSEX])
    }

    /// Query the board for available analog pins.
    fn query_analog_mapping(&mut self) -> Result<(), Error> {
        self.write(&[START_SYSEX, ANALOG_MAPPING_QUERY, END_SYSEX])
    }

    // ########################################
    // Read/Write on pins

    /// Write `level` to the analog `pin`.
    fn analog_write(&mut self, pin: i32, level: i32) -> Result<(), Error> {
        self.hardware_mut().pins[pin as usize].value = level;
        self.write(&[
            ANALOG_MESSAGE | pin as u8,
            level as u8 & SYSEX_REALTIME,
            (level >> 7) as u8 & SYSEX_REALTIME,
        ])
    }
    /// Write `level` to the digital `pin`.
    fn digital_write(&mut self, pin: i32, level: i32) -> Result<(), Error> {
        let port = (pin as f64 / 8f64).floor() as usize;
        let mut value = 0i32;
        let mut i = 0;

        self.hardware_mut().pins[pin as usize].value = level;

        while i < 8 {
            if self.hardware().pins[8 * port + i].value != 0 {
                value |= 1 << i
            }
            i += 1;
        }

        self.write(&[
            DIGITAL_MESSAGE | port as u8,
            value as u8 & SYSEX_REALTIME,
            (value >> 7) as u8 & SYSEX_REALTIME,
        ])
    }

    /// Set the analog reporting `state` of the specified `pin`.
    fn report_analog(&mut self, pin: i32, state: i32) -> Result<(), Error> {
        self.write(&[REPORT_ANALOG | pin as u8, state as u8])
    }
    /// Set the digital reporting `state` of the specified `pin`.
    fn report_digital(&mut self, pin: i32, state: i32) -> Result<(), Error> {
        self.write(&[REPORT_DIGITAL | pin as u8, state as u8])
    }
    /// Set the `mode` of the specified `pin`.
    fn set_pin_mode(&mut self, pin: i32, mode: u8) -> Result<(), Error> {
        self.hardware_mut().pins[pin as usize].supported_modes = vec![mode];
        self.write(&[SET_PIN_MODE, pin as u8, mode])
    }

    // ########################################
    // I2C

    /// Configure the `delay` in microseconds for I2C devices that require a delay between when the
    /// register is written to and the data in that register can be read.
    fn i2c_config(&mut self, delay: i32) -> Result<(), Error> {
        self.write(&[
            START_SYSEX,
            I2C_CONFIG,
            (delay & 0xFF) as u8,
            (delay >> 8 & 0xFF) as u8,
            END_SYSEX,
        ])
    }
    /// Read `size` bytes from I2C device at the specified `address`.
    fn i2c_read(&mut self, address: i32, size: i32) -> Result<(), Error> {
        self.write(&[
            START_SYSEX,
            I2C_REQUEST,
            address as u8,
            I2C_READ << 3,
            (size as u8) & SYSEX_REALTIME,
            (size >> 7) as u8 & SYSEX_REALTIME,
            END_SYSEX,
        ])
    }
    /// Write `data` to the I2C device at the specified `address`.
    fn i2c_write(&mut self, address: i32, data: &[u8]) -> Result<(), Error> {
        let mut buf = vec![START_SYSEX, I2C_REQUEST, address as u8, I2C_WRITE << 3];

        for i in data.iter() {
            buf.push(i & SYSEX_REALTIME);
            buf.push(((*i as i32) >> 7) as u8 & SYSEX_REALTIME);
        }

        buf.push(END_SYSEX);

        self.write(&buf)
    }

    // ########################################
    // FIRMATA main function.

    /// Read from the protocol, parse and return its type.
    /// The following method should use Firmata Protocol such as defined here:  
    /// https://github.com/firmata/protocol
    fn read_and_decode(&mut self) -> Result<Message, Error> {
        let mut buf = vec![0; 3];
        self.read_exact(&mut buf)?;

        match buf[0] {
            REPORT_VERSION => self.handle_report_version(&buf),
            ANALOG_MESSAGE..=ANALOG_MESSAGE_BOUND => self.handle_analog_message(&buf),
            DIGITAL_MESSAGE..=DIGITAL_MESSAGE_BOUND => self.handle_digital_message(&buf),
            START_SYSEX => self.handle_sysex_message(&mut buf),
            _ => Err(BadByte { byte: buf[0] }),
        }
    }

    fn handle_report_version(&mut self, buf: &[u8]) -> Result<Message, Error> {
        self.hardware_mut().protocol_version = format!("{:o}.{:o}", buf[1], buf[2]);
        Ok(Message::ProtocolVersion)
    }

    fn handle_analog_message(&mut self, buf: &[u8]) -> Result<Message, Error> {
        if buf.len() < 3 {
            return Err(MessageTooShort);
        }
        let pin = ((buf[0] as i32) & 0x0F) + 14;
        let value = (buf[1] as i32) | ((buf[2] as i32) << 7);
        if self.hardware().pins.len() as i32 > pin {
            self.hardware_mut().pins[pin as usize].value = value;
        }
        Ok(Message::Analog)
    }

    fn handle_digital_message(&mut self, buf: &[u8]) -> Result<Message, Error> {
        if buf.len() < 3 {
            return Err(MessageTooShort);
        }
        let port = (buf[0] as i32) & 0x0F;
        let value = (buf[1] as i32) | ((buf[2] as i32) << 7);

        for i in 0..8 {
            let pin = (8 * port) + i;
            let mode: u8 = self.hardware().pins[pin as usize].mode;
            if self.hardware().pins.len() as i32 > pin && mode == PIN_MODE_INPUT {
                self.hardware_mut().pins[pin as usize].value = (value >> (i & 0x07)) & 0x01;
            }
        }
        Ok(Message::Digital)
    }

    fn handle_sysex_message(&mut self, buf: &mut Vec<u8>) -> Result<Message, Error> {
        loop {
            // Read until END_SYSEX.
            let mut byte = [0];
            self.read_exact(&mut byte)?;
            buf.push(byte[0]);
            if byte[0] == END_SYSEX {
                break;
            }
        }
        match buf[1] {
            END_SYSEX => Ok(Message::EmptyResponse),
            ANALOG_MAPPING_RESPONSE => self.handle_analog_mapping_response(buf),
            CAPABILITY_RESPONSE => self.handle_capability_response(buf),
            REPORT_FIRMWARE => self.handle_report_firmware(buf),
            I2C_REPLY => self.handle_i2c_reply(buf),
            PIN_STATE_RESPONSE => self.handle_pin_state_response(buf),
            _ => Err(UnknownSysEx { code: buf[1] }),
        }
    }

    fn handle_analog_mapping_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut i = 2;
        let upper = (buf.len() - 1).min(self.hardware().pins.len() + 2);
        while i < upper {
            if buf[i] != 127u8 {
                let pin = &mut self.hardware_mut().pins[i - 2];
                pin.mode = PIN_MODE_ANALOG;
                pin.supported_modes = vec![PIN_MODE_ANALOG];
                pin.resolution = DEFAULT_ANALOG_RESOLUTION;
            }
            i += 1;
        }
        Ok(Message::AnalogMappingResponse)
    }

    fn handle_capability_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut id = 0;
        let mut i = 2;
        self.hardware_mut().pins = vec![];
        let mut supported_modes = vec![];
        let mut resolution = None;
        while i < buf.len() - 1 {
            if buf[i] == 127u8 {
                self.hardware_mut().pins.push(Pin {
                    id,
                    mode: buf[i],
                    supported_modes: vec![],
                    resolution: buf[i + 1],
                    value: 0,
                });
                id += 1;
                i += 1;
            } else {
                supported_modes.push(buf[i]);
                if resolution.is_none() {
                    resolution.replace(buf[i + 1]);
                }
                i += 2;
            }
        }
        Ok(Message::CapabilityResponse)
    }

    fn handle_report_firmware(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let major = *buf.get(2).ok_or(MessageTooShort)?;
        let minor = *buf.get(3).ok_or(MessageTooShort)?;
        self.hardware_mut().firmware_version = format!("{:o}.{:o}", major, minor);
        if 4 < buf.len() - 1 {
            self.hardware_mut().firmware_name = std::str::from_utf8(&buf[4..buf.len() - 1])
                .with_context(|_| Utf8Snafu)?
                .to_string();
        }
        Ok(Message::ReportFirmware)
    }

    fn handle_i2c_reply(&mut self, buf: &[u8]) -> Result<Message, Error> {
        if buf.len() < 8 {
            return Err(MessageTooShort);
        }
        let mut reply = I2CReply {
            address: (buf[2] as i32) | ((buf[3] as i32) << 7),
            register: (buf[4] as i32) | ((buf[5] as i32) << 7),
            data: vec![buf[6] | buf[7] << 7],
        };
        let mut i = 8;
        while i < buf.len() - 1 {
            if buf[i] == 0xF7 {
                break;
            }
            if i + 2 > buf.len() {
                break;
            }
            reply.data.push(buf[i] | buf[i + 1] << 7);
            i += 2;
        }
        self.hardware_mut().i2c_data.push(reply);
        Ok(Message::I2CReply)
    }

    fn handle_pin_state_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let pin_index = buf[2] as usize;
        if buf[3] == END_SYSEX {
            return Ok(Message::PinStateResponse);
        }
        let pin = &mut self.hardware_mut().pins[pin_index];
        pin.supported_modes = vec![buf[3]];
        pin.value = buf[4] as i32;
        Ok(Message::PinStateResponse)
    }
}

impl Display for Box<dyn Protocol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "firmware={}, version={}, protocol={}, connection={:?}",
            self.firmware_name,
            self.firmware_version,
            self.get_protocol_name(),
            self.get_protocol_details()
        )
    }
}

impl Deref for dyn Protocol {
    type Target = ProtocolHardware;

    fn deref(&self) -> &Self::Target {
        self.hardware()
    }
}

impl DerefMut for dyn Protocol {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.hardware_mut()
    }
}
