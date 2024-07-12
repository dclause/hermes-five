use std::any::type_name;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;
use snafu::ResultExt;

// Flatten export
pub use crate::protocols::constants::*;
pub use crate::protocols::Error::*;
pub use crate::protocols::errors::*;
pub use crate::protocols::flavor::*;
pub use crate::protocols::i2c_reply::I2CReply;
pub use crate::protocols::pins::*;
pub use crate::protocols::protocol::*;

pub mod constants;
mod errors;
mod flavor;
mod i2c_reply;
mod pins;
mod protocol;

// Makes a Box<dyn Protocol> clone (used for Board cloning).
dyn_clone::clone_trait_object!(Protocol);

/// Defines the trait all protocols must implement.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Protocol: DynClone + Send + Sync + Debug {
    // ########################################
    // Inner data related functions

    /// Retrieve the internal hardware.
    fn hardware(&self) -> &ProtocolHardware;
    fn hardware_mut(&mut self) -> &mut ProtocolHardware;

    /// Returns the protocol name (used for Display only)
    fn get_protocol_name(&self) -> &'static str {
        type_name::<Self>().split("::").last().unwrap()
    }

    /// Returns the protocol internal details (used for Display only)
    fn get_protocol_details(&self) -> String {
        String::from("()")
    }

    // ########################################
    // Functions specifically bound to the protocol.

    /// Open the communication using the underlying protocol.
    fn open(&mut self) -> Result<(), Error>;
    /// Gracefully shuts down the communication.
    fn close(&mut self) -> Result<(), Error>;
    /// Write to the internal connection. For more details see [`std::io::Write::write`].
    fn write(&mut self, buf: &[u8]) -> Result<(), Error>;
    /// Read from the internal connection. For more details see [`std::io::Read::read_exact`].
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error>;

    // ########################################
    // Protocol related functions

    /// Starts a conversation with the board: validate the firmware version and...
    fn handshake(&mut self) -> Result<(), Error> {
        self.query_firmware()?;
        self.read_and_decode()?;
        self.query_capabilities()?;
        self.read_and_decode()?;
        self.query_analog_mapping()?;
        self.read_and_decode()?;
        self.report_digital(0, true)?;
        self.report_digital(1, true)?;
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
    ///
    /// Send an ANALOG_MESSAGE (0xE0 - set analog value).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#message-types
    fn analog_write(&mut self, pin: u16, level: u16) -> Result<(), Error> {
        {
            let mut lock = self.hardware().write();
            lock.pins
                .get_mut(pin as usize)
                .ok_or(UnknownPin { pin })?
                .value = level;
        }
        self.write(&[
            ANALOG_MESSAGE | pin as u8,
            level as u8 & SYSEX_REALTIME,
            (level >> 7) as u8 & SYSEX_REALTIME,
        ])
    }

    /// Write `level` to the digital `pin`.
    ///
    /// Send an DIGITAL_MESSAGE (0x90 - set analog value).   
    /// https://github.com/firmata/protocol/blob/master/protocol.md#message-types
    fn digital_write(&mut self, pin: u16, level: bool) -> Result<(), Error> {
        let port = (pin / 8) as u8;
        let mut value: u16 = 0;
        let mut i = 0;

        {
            let mut lock = self.hardware().write();
            let _pin = lock.get_pin_mut(pin)?;

            // Check if mode is oK.
            _pin.check_current_mode(PinModeId::OUTPUT)?;

            // Store the value we will write to the current pin.
            lock.get_pin_mut(pin)?.value = u16::from(level);

            // Loop through all 8 pins of the current "port" to concatenate their value.
            // For instance 01100000 will set to 1 the pin 1 and 2 or current port.
            while i < 8 {
                if lock.get_pin_mut((8 * port + i) as u16)?.value != 0 {
                    value |= 1 << i
                }
                i += 1;
            }
        }

        self.write(&[
            DIGITAL_MESSAGE | port as u8,
            value as u8 & SYSEX_REALTIME,
            (value >> 7) as u8 & SYSEX_REALTIME,
        ])
    }

    /// Set the analog reporting `state` of the specified `pin`.
    ///
    /// This will activate the reporting of the pin (hence the pin will send us its value periodically)
    /// https://github.com/firmata/protocol/blob/master/protocol.md
    fn report_analog(&mut self, channel: u8, state: bool) -> Result<(), Error> {
        self.write(&[REPORT_ANALOG | channel, u8::from(state)])
    }

    /// Set the digital reporting `state` of all pins in specified `port`.
    ///
    /// This will activate the reporting of all pins in port (hence the pin will send us its value periodically)
    /// https://github.com/firmata/protocol/blob/master/protocol.md
    fn report_digital(&mut self, port: u8, state: bool) -> Result<(), Error> {
        self.write(&[REPORT_DIGITAL | port, u8::from(state)])
    }

    /// Set the `mode` of the specified `pin`.
    ///
    /// https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion
    fn set_pin_mode(&mut self, pin: u16, mode: PinModeId) -> Result<(), Error> {
        {
            let mut lock = self.hardware().write();
            let mut _pin = lock.get_pin_mut(pin)?;
            let _mode = _pin.get_plausible_mode(mode)?;
            _pin.mode = _mode;
        }

        self.write(&[SET_PIN_MODE, pin as u8, mode as u8])
    }

    // ########################################
    // I2C

    /// Configure the `delay` in microseconds for I2C devices that require a delay between when the
    /// register is written to and the data in that register can be read.
    fn i2c_config(&mut self, delay: u16) -> Result<(), Error> {
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

        for &i in data.iter() {
            buf.push(i & SYSEX_REALTIME);
            buf.push(((i as i32) >> 7) as u8 & SYSEX_REALTIME);
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
            REPORT_VERSION_RESPONSE => self.handle_report_version(&buf),
            ANALOG_MESSAGE..=ANALOG_MESSAGE_BOUND => self.handle_analog_message(&buf),
            DIGITAL_MESSAGE..=DIGITAL_MESSAGE_BOUND => self.handle_digital_message(&buf),
            START_SYSEX => self.handle_sysex_message(&mut buf),
            _ => Err(BadByte { byte: buf[0] }),
        }
    }

    /// Handle a REPORT_VERSION_RESPONSE message (0xF9 - return the firmware version).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#message-types
    fn handle_report_version(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut lock = self.hardware().write();
        lock.protocol_version = format!("{:o}.{:o}", buf[1], buf[2]);
        Ok(Message::ProtocolVersion)
    }

    fn handle_analog_message(&mut self, buf: &[u8]) -> Result<Message, Error> {
        // if buf.len() < 3 {
        //     return Err(MessageTooShort);
        // }
        // let pin = ((buf[0] as i32) & 0x0F) + 14;
        // let value = (buf[1] as i32) | ((buf[2] as i32) << 7);
        // if self.hardware().pins.len() as i32 > pin {
        //     self.hardware_mut().pins[pin as usize].value = value;
        // }
        Ok(Message::Analog)
    }

    fn handle_digital_message(&mut self, buf: &[u8]) -> Result<Message, Error> {
        // if buf.len() < 3 {
        //     return Err(MessageTooShort);
        // }
        // let port = (buf[0] as i32) & 0x0F;
        // let value = (buf[1] as i32) | ((buf[2] as i32) << 7);
        //
        // for i in 0..8 {
        //     let pin = (8 * port) + i;
        //     let mode: PinModeId = self.hardware().pins[pin as usize].mode;
        //     if self.hardware().pins.len() as i32 > pin && mode == PinModeId::INPUT {
        //         self.hardware_mut().pins[pin as usize].value = (value >> (i & 0x07)) & 0x01;
        //     }
        // }
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

    /// Handle an ANALOG_MAPPING_RESPONSE message (0x6A - reply with analog pins mapping info).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#analog-mapping-query
    fn handle_analog_mapping_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut lock = self.hardware().write();
        let mut i = 2;
        while buf[i] != END_SYSEX {
            if buf[i] != SYSEX_REALTIME {
                let pin = &mut lock.get_pin_mut((i - 2) as u16)?;
                pin.mode = pin.get_plausible_mode(PinModeId::ANALOG)?.clone();
                pin.channel = Some(buf[i]);
            }
            i += 1;
        }
        Ok(Message::AnalogMappingResponse)
    }

    /// Handle a CAPABILITY_RESPONSE message (0x6C - reply with supported modes and resolution)
    /// https://github.com/firmata/protocol/blob/master/protocol.md#capability-query
    fn handle_capability_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut id = 0;
        let mut i = 2;
        let mut lock = self.hardware().write();
        lock.pins = vec![];

        while i < buf.len() - 1 {
            let mut supported_modes: Vec<PinMode> = vec![];

            while buf[i] != 127u8 {
                supported_modes.push(PinMode {
                    id: PinModeId::from_u8(buf[i])?,
                    resolution: buf[i + 1],
                });
                i += 2;
            }

            let mut pin = Pin {
                id,
                mode: PinMode::default(),
                supported_modes,
                value: 0,
                channel: None,
            };
            if !pin.supported_modes.is_empty() {
                pin.mode = pin.supported_modes.first().unwrap().clone();
            }
            lock.pins.push(pin);

            i += 1;
            id += 1;
        }
        Ok(Message::CapabilityResponse)
    }

    /// Handle a REPORT_FIRMWARE message (0x79 - report name and version of the firmware).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#query-firmware-name-and-version
    fn handle_report_firmware(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let major = *buf.get(2).ok_or(MessageTooShort)?;
        let minor = *buf.get(3).ok_or(MessageTooShort)?;
        let mut lock = self.hardware().write();
        lock.firmware_version = format!("{:o}.{:o}", major, minor);
        if buf.len() > 4 {
            lock.firmware_name = std::str::from_utf8(&buf[4..buf.len() - 1])
                .with_context(|_| Utf8Snafu)?
                .to_string();
        }
        Ok(Message::ReportFirmware)
    }

    fn handle_i2c_reply(&mut self, buf: &[u8]) -> Result<Message, Error> {
        // if buf.len() < 8 {
        //     return Err(MessageTooShort);
        // }
        // let mut reply = I2CReply {
        //     address: (buf[2] as i32) | ((buf[3] as i32) << 7),
        //     register: (buf[4] as i32) | ((buf[5] as i32) << 7),
        //     data: vec![buf[6] | buf[7] << 7],
        // };
        // let mut i = 8;
        // while i < buf.len() - 1 {
        //     if buf[i] == 0xF7 {
        //         break;
        //     }
        //     if i + 2 > buf.len() {
        //         break;
        //     }
        //     reply.data.push(buf[i] | buf[i + 1] << 7);
        //     i += 2;
        // }
        // self.hardware_mut().i2c_data.push(reply);
        Ok(Message::I2CReply)
    }

    fn handle_pin_state_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        // let pin_index = buf[2] as usize;
        // if buf.len() < 4 || buf[3] == END_SYSEX {
        //     return Ok(Message::PinStateResponse);
        // }
        // let pin = &mut self.hardware_mut().pins[pin_index];
        // pin.supported_modes = vec![PinModeId::from_u8(buf[3])?];
        // pin.value = buf[4] as i32;
        Ok(Message::PinStateResponse)
    }
}

impl Display for Box<dyn Protocol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lock = self.hardware().read();
        write!(
            f,
            "firmware={}, version={}, protocol={}, connection={:?}",
            lock.firmware_name,
            lock.firmware_version,
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

#[cfg(test)]
mod tests {

    // @todo Implement tests

    // #[test]
    // fn test_handshake() {
    //     let mut protocol = MockProtocol {
    //         hardware: ProtocolHardware::default(),
    //     };
    //
    // assert_eq!(protocol.get_protocol_name(), "MockProtocol");
    //
    //     let handshake = protocol.handshake();
    //     assert!(
    //         handshake.is_ok(),
    //         "Handshake error: {}",
    //         handshake.unwrap_err()
    //     );
    //
    //     assert_eq!(protocol.hardware.firmware_version, "2.5");
    // }
    //
    // #[test]
    // fn test_analog_write() {
    //     let mut protocol = MockProtocol {
    //         hardware: ProtocolHardware::default(),
    //     };
    //
    //     assert!(protocol.analog_write(0, 255).is_ok());
    //
    //     // Add more assertions based on your specific protocol behavior
    //     assert_eq!(protocol.hardware.pins[0].value, 255);
    // }
    //
    // #[test]
    // fn test_digital_write() {
    //     let mut protocol = MockProtocol {
    //         hardware: ProtocolHardware::default(),
    //     };
    //
    //     assert!(protocol.digital_write(7, 1).is_ok());
    //
    //     // Add more assertions based on your specific protocol behavior
    //     assert_eq!(protocol.hardware.pins[7].value, 1);
    // }
}
