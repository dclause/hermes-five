use std::any::type_name;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use dyn_clone::DynClone;
use parking_lot::RwLock;

use crate::errors::HardwareError::IncompatibleMode;
use crate::errors::ProtocolError::{MessageTooShort, UnexpectedData};
use crate::errors::*;
pub use crate::protocols::constants::*;
pub use crate::protocols::flavor::*;
pub use crate::protocols::hardware::*;
pub use crate::protocols::i2c_reply::I2CReply;
pub use crate::protocols::pins::*;
use crate::utils::Range;

pub mod constants;
mod flavor;
mod hardware;
mod i2c_reply;
mod pins;

// Makes a Box<dyn Protocol> clone (used for Board cloning).
dyn_clone::clone_trait_object!(Protocol);

/// Defines the trait all protocols must implement.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Protocol: DynClone + Send + Sync + Debug {
    // ########################################
    // Inner data related functions

    /// Retrieve the internal hardware.
    fn get_hardware(&self) -> &Arc<RwLock<Hardware>>;

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
            let mut lock = self.get_hardware().write();
            lock.get_pin_mut(pin)?.value = level;
        }
        self.write(&[
            ANALOG_MESSAGE | pin as u8,
            level as u8 & SYSEX_REALTIME,
            (level >> 7) as u8 & SYSEX_REALTIME,
        ])
    }

    /// Write `level` to the digital `pin`.
    ///
    /// Send an DIGITAL_MESSAGE (0x90 - set digital value).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#message-types
    fn digital_write(&mut self, pin: u16, level: bool) -> Result<(), Error> {
        let port = (pin / 8) as u8;
        let mut value: u16 = 0;
        let mut i = 0;

        {
            let mut lock = self.get_hardware().write();

            // Check if pin exists
            let pin_instance = lock.get_pin_mut(pin)?;

            // Check if mode is oK.
            pin_instance.validate_current_mode(PinModeId::OUTPUT)?;

            // Store the value we will write to the current pin.
            pin_instance.value = u16::from(level);

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
            let mut lock = self.get_hardware().write();
            let mut pin_instance = lock.get_pin_mut(pin)?;
            let _mode = pin_instance.supports_mode(mode).ok_or(IncompatibleMode {
                pin,
                mode,
                context: "try to set pin mode",
            })?;
            pin_instance.mode = _mode;
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
            (delay as u8) & SYSEX_REALTIME,
            (delay >> 7) as u8 & SYSEX_REALTIME,
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
    // SERVO

    /// Sends a SERVO_CONFIG command (0x70 - configure servo)
    /// https://github.com/firmata/protocol/blob/master/servos.md
    fn servo_config(&mut self, pin: u16, pwm_range: Range<u16>) -> Result<(), Error> {
        self.write(&[
            START_SYSEX,
            SERVO_CONFIG,
            pin as u8,
            pwm_range.start as u8 & SYSEX_REALTIME,
            (pwm_range.start >> 7) as u8 & SYSEX_REALTIME,
            pwm_range.end as u8 & SYSEX_REALTIME,
            (pwm_range.end >> 7) as u8 & SYSEX_REALTIME,
            END_SYSEX,
        ])
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
            _ => Err(Error::from(UnexpectedData)),
        }
    }

    /// Handle a REPORT_VERSION_RESPONSE message (0xF9 - return the firmware version).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#message-types
    fn handle_report_version(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut lock = self.get_hardware().write();
        lock.protocol_version = format!("{}.{}", buf[1], buf[2]);
        Ok(Message::ReportProtocolVersion)
    }

    /// Handle an ANALOG_MESSAGE message (0xE0 - report state of an analog pin)
    /// https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion
    fn handle_analog_message(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let pin = (buf[0] as u16 & 0x0F) + 14;
        let value = (buf[1] as u16) | ((buf[2] as u16) << 7);
        self.get_hardware().write().get_pin_mut(pin)?.value = value;
        Ok(Message::Analog)
    }

    /// Handle a DIGITAL_MESSAGE message (0x90 - report state of a digital pin/port)
    /// https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion
    fn handle_digital_message(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let port = (buf[0] as u16) & 0x0F;
        let value = (buf[1] as u16) | ((buf[2] as u16) << 7);

        for i in 0..8 {
            let pin = (8 * port) + i;
            let mode: PinModeId = self.get_hardware().read().get_pin(pin)?.mode.id;
            if mode == PinModeId::INPUT {
                self.get_hardware().write().get_pin_mut(pin)?.value = (value >> (i & 0x07)) & 0x01;
            }
        }
        Ok(Message::Digital)
    }

    /// Handle a START_SYSEX message: dispatch to various message/command/response using the sysex format.
    /// https://github.com/firmata/protocol/blob/master/protocol.md#sysex-message-format
    fn handle_sysex_message(&mut self, buf: &mut Vec<u8>) -> Result<Message, Error> {
        if buf[1] == END_SYSEX || buf[2] == END_SYSEX {
            return Ok(Message::EmptyResponse);
        }

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
            ANALOG_MAPPING_RESPONSE => self.handle_analog_mapping_response(buf),
            CAPABILITY_RESPONSE => self.handle_capability_response(buf),
            REPORT_FIRMWARE => self.handle_report_firmware(buf),
            I2C_REPLY => self.handle_i2c_reply(buf),
            PIN_STATE_RESPONSE => self.handle_pin_state_response(buf),
            _ => Err(Error::from(UnexpectedData)),
        }
    }

    /// Handle an ANALOG_MAPPING_RESPONSE message (0x6A - reply with analog pins mapping info).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#analog-mapping-query
    fn handle_analog_mapping_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut lock = self.get_hardware().write();
        let mut i = 2;
        while buf[i] != END_SYSEX {
            if buf[i] != SYSEX_REALTIME {
                let pin = &mut lock.get_pin_mut((i - 2) as u16)?;
                pin.mode = pin
                    .supports_mode(PinModeId::ANALOG)
                    .ok_or(IncompatibleMode {
                        pin: (i - 2) as u16,
                        mode: PinModeId::ANALOG,
                        context: "handle_analog_mapping_response",
                    })?;
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
        let mut lock = self.get_hardware().write();
        lock.pins = HashMap::new();

        while buf[i] != END_SYSEX {
            let mut supported_modes: Vec<PinMode> = vec![];

            while buf[i] != SYSEX_REALTIME {
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
            lock.pins.insert(pin.id, pin);

            i += 1;
            id += 1;
        }
        Ok(Message::CapabilityResponse)
    }

    /// Handle a REPORT_FIRMWARE message (0x79 - report name and version of the firmware).
    /// https://github.com/firmata/protocol/blob/master/protocol.md#query-firmware-name-and-version
    fn handle_report_firmware(&mut self, buf: &[u8]) -> Result<Message, Error> {
        if buf.len() < 5 {
            return Err(Error::from(MessageTooShort {
                operation: "handle_report_firmware",
                expected: 5,
                received: buf.len(),
            }));
        }
        let major = buf[2];
        let minor = buf[3];
        let mut lock = self.get_hardware().write();
        lock.firmware_version = format!("{}.{}", major, minor);
        if buf.len() > 5 {
            lock.firmware_name = std::str::from_utf8(&buf[4..buf.len() - 1])?.to_string();
        }
        Ok(Message::ReportFirmwareVersion)
    }

    /// Handle an I2C_REPLY message (0x6E - read and decode an i2c message)
    /// https://github.com/firmata/protocol/blob/master/i2c.md
    fn handle_i2c_reply(&mut self, buf: &[u8]) -> Result<Message, Error> {
        if buf.len() < 8 {
            return Err(Error::from(MessageTooShort {
                operation: "handle_i2c_reply",
                expected: 9,
                received: buf.len(),
            }));
        }
        let mut reply = I2CReply {
            address: (buf[2] as u16) | ((buf[3] as u16) << 7),
            register: (buf[4] as u16) | ((buf[5] as u16) << 7),
            data: vec![(buf[6] as u16) | (buf[7] as u16) << 7],
        };
        let mut i = 8;
        while buf[i] != END_SYSEX {
            reply.data.push((buf[i] as u16) | (buf[i + 1] as u16) << 7);
            i += 2;
        }
        self.get_hardware().write().i2c_data.push(reply);
        Ok(Message::I2CReply)
    }

    /// Handle a PIN_STATE_RESPONSE message (0x6E - report pin current mode and state)
    /// https://github.com/firmata/protocol/blob/master/protocol.md#pin-state-query
    fn handle_pin_state_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let pin = buf[2] as u16;
        if buf.len() < 4 || buf[3] == END_SYSEX {
            return Err(Error::from(MessageTooShort {
                operation: "handle_pin_state_response",
                expected: 5,
                received: buf.len(),
            }));
        }

        let mut lock = self.get_hardware().write();
        let pin = lock.get_pin_mut(pin)?;
        // Check if the state announce by the protocol is plausible and fetch it.
        let mode = PinModeId::from_u8(buf[3])?;
        let current_state = pin.supports_mode(mode).unwrap();
        pin.mode = current_state;

        let mut i = 4;
        let mut value: usize = 0;
        while buf[i] != END_SYSEX {
            // Shift value by 7 bits and combine with the next 7 bits
            value = (value << 7) | ((buf[i] as usize) & 0x7F);
            i += 1;
        }
        pin.value = value as u16;
        Ok(Message::PinStateResponse)
    }
}

impl Display for Box<dyn Protocol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lock = self.get_hardware().read();
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

#[cfg(test)]
mod tests {
    use crate::protocols::{Message, PinModeId, Protocol};
    use crate::tests::mocks::protocol::MockProtocol;
    use crate::utils::Range;

    fn format_as_hex(slice: &[u8]) -> String {
        slice
            .iter()
            .map(|byte| format!("0x{:02X}", byte))
            .collect::<Vec<String>>()
            .join(", ")
    }

    #[test]
    fn test_handshake() {
        let mut protocol = MockProtocol::default();
        protocol.index = 10;
        // Result for query firmware
        protocol.buf[10..15].copy_from_slice(&[0xF0, 0x79, 0x01, 0x0C, 0xF7]);
        // Result for report capabilities
        protocol.buf[15..26].copy_from_slice(&[
            0xF0, 0x6C, 0x00, 0x08, 0x7F, 0x00, 0x08, 0x01, 0x08, 0x7F, 0xF7,
        ]);
        // Result for analog mapping
        protocol.buf[26..32].copy_from_slice(&[0xF0, 0x6A, 0x7F, 0x7F, 0x7F, 0xF7]);

        let result = protocol.handshake();
        assert!(result.is_ok(), "{:?}", result);
    }

    #[test]
    fn test_analog_write() {
        let mut protocol = MockProtocol::default();
        let result = protocol.analog_write(0, 170);
        assert!(result.is_ok(), "{:?}", result);
        assert!(
            protocol.buf.starts_with(&[0xE0, 0x2A, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..3])
        );
        {
            let lock = protocol.get_hardware().read();
            let pin = lock.get_pin(0).unwrap();
            assert_eq!(pin.value, 170, "Pin value updated");
        }

        let result = protocol.analog_write(5, 0);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Unknown pin 5."
        );
    }

    #[test]
    fn test_digital_write() {
        let mut protocol = MockProtocol::default();
        let result = protocol.digital_write(13, true);
        assert!(result.is_ok(), "{:?}", result);
        assert!(
            protocol.buf.starts_with(&[0x91, 0x7F, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..3])
        );

        {
            let lock = protocol.get_hardware().read();
            let pin = lock.get_pin(13).unwrap();
            assert_eq!(pin.value, 1);
            let pin = lock.get_pin(11).unwrap();
            assert_eq!(pin.value, 11, "Other pin value does not change");
        }

        let result = protocol.digital_write(7, true);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Unknown pin 7."
        );
    }

    #[test]
    fn test_set_pin_mode() {
        let mut protocol = MockProtocol::default();

        {
            let lock = protocol.get_hardware().read();
            let pin = lock.get_pin(8).unwrap();
            assert_eq!(pin.mode.id, PinModeId::PWM);
        }

        let result = protocol.set_pin_mode(8, PinModeId::OUTPUT);
        assert!(result.is_ok(), "{:?}", result);
        assert!(
            protocol.buf.starts_with(&[0xF4, 0x08, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..3])
        );

        {
            let lock = protocol.get_hardware().read();
            let pin = lock.get_pin(8).unwrap();
            assert_eq!(pin.mode.id, PinModeId::OUTPUT);
        }

        let result = protocol.set_pin_mode(8, PinModeId::SHIFT);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Pin (8) not compatible with mode (SHIFT) - try to set pin mode."
        );
    }

    #[test]
    fn test_servo_config() {
        let mut protocol = MockProtocol::default();
        let result = protocol.servo_config(8, Range::from([500, 2500]));
        assert!(
            result.is_ok(),
            "Servo config error: {:?}",
            result.unwrap_err()
        );
        assert!(
            protocol
                .buf
                .starts_with(&[0xF0, 0x70, 0x08, 0x74, 0x03, 0x44, 0x13, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..8])
        );
    }

    #[test]
    fn test_query_firmware() {
        let mut protocol = MockProtocol::default();
        let result = protocol.query_firmware();
        assert!(result.is_ok(), "{:?}", result);
        assert!(
            protocol.buf.starts_with(&[0xF0, 0x79, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..3])
        );
    }

    #[test]
    fn test_query_capabilities() {
        let mut protocol = MockProtocol::default();
        let result = protocol.query_capabilities();
        assert!(
            result.is_ok(),
            "Query capabilities: {:?}",
            result.unwrap_err()
        );
        assert!(
            protocol.buf.starts_with(&[0xF0, 0x6B, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..3])
        );
    }

    #[test]
    fn test_query_analog_mapping() {
        let mut protocol = MockProtocol::default();
        let result = protocol.query_analog_mapping();
        assert!(
            result.is_ok(),
            "Query analog mapping: {:?}",
            result.unwrap_err()
        );
        assert!(
            protocol.buf.starts_with(&[0xF0, 0x69, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..3])
        );
    }

    #[test]
    fn test_report_analog() {
        let mut protocol = MockProtocol::default();
        let result = protocol.report_analog(1, true);
        assert!(result.is_ok(), "{:?}", result);
        assert!(
            protocol.buf.starts_with(&[0xC1, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..2])
        );
    }

    #[test]
    fn test_report_digital() {
        let mut protocol = MockProtocol::default();
        let result = protocol.report_digital(1, true);
        assert!(result.is_ok(), "{:?}", result);
        assert!(
            protocol.buf.starts_with(&[0xD1, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..2])
        );
    }

    #[test]
    fn test_handle_report_version() {
        let mut protocol = MockProtocol::default();
        protocol.buf[..3].copy_from_slice(&[0xF9, 0x01, 0x19]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report version: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::ReportProtocolVersion);
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(lock.to_owned().protocol_version, "1.25");
        }
    }

    #[test]
    fn test_handle_analog_message() {
        let mut protocol = MockProtocol::default();
        protocol.buf[..3].copy_from_slice(&[0xE1, 0xDE, 0x00]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report version: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::Analog);
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(lock.to_owned().get_pin(15).unwrap().value, 222);
        }
    }

    #[test]
    fn test_handle_digital_message() {
        let mut protocol = MockProtocol::default();
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(lock.to_owned().get_pin(10).unwrap().value, 10);
            assert_eq!(lock.to_owned().get_pin(12).unwrap().value, 12);
        }

        protocol.buf[..3].copy_from_slice(&[0x91, 0x00, 0x00]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report version: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::Digital);
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(lock.to_owned().get_pin(10).unwrap().value, 0);
            assert_eq!(lock.to_owned().get_pin(12).unwrap().value, 12);
        }
    }

    #[test]
    fn test_handle_empty_sysex() {
        // Unexpected data when the first byte received in not a valid command.
        let mut protocol = MockProtocol::default();
        protocol.buf[..1].copy_from_slice(&[0x11]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_err(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(
            result.err().unwrap().to_string(),
            "Protocol error: Unexpected data received."
        );

        // Unexpected data when the first byte is a sysex, the size is plausible,
        // but the second is not a valid sysex command.
        let mut protocol = MockProtocol::default();
        protocol.buf[..4].copy_from_slice(&[0xF0, 0x11, 0x11, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_err(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(
            result.err().unwrap().to_string(),
            "Protocol error: Unexpected data received."
        );

        // Empty command error when a sysex is received and closed immediately.
        protocol.index = 0;
        protocol.buf[..2].copy_from_slice(&[0xF0, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::EmptyResponse);
    }

    #[test]
    fn test_handle_analog_mapping_response() {
        let mut protocol = MockProtocol::default();
        protocol.buf[..6].copy_from_slice(&[0xF0, 0x6A, 0x01, 0x7F, 0x7F, 0xF7]);
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(lock.to_owned().get_pin(0).unwrap().channel, None);
        }
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::AnalogMappingResponse);
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(lock.to_owned().get_pin(0).unwrap().channel, Some(1));
        }

        // Unsupported possible data
        let mut protocol = MockProtocol::default();
        protocol.buf[..6].copy_from_slice(&[0xF0, 0x6A, 0x01, 0x01, 0x01, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Pin (2) not compatible with mode (ANALOG) - handle_analog_mapping_response."
        );
    }

    #[test]
    fn test_handle_capability_response() {
        let mut protocol = MockProtocol::default();
        protocol.buf[..11].copy_from_slice(&[
            0xF0, 0x6C, 0x00, 0x08, 0x7F, 0x00, 0x08, 0x01, 0x08, 0x7F, 0xF7,
        ]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::CapabilityResponse);
        {
            let lock = protocol.get_hardware().read();
            let hardware = lock.to_owned();
            assert_eq!(hardware.pins.len(), 2, "{:?}", hardware.pins);
            assert_eq!(hardware.get_pin(0).unwrap().supported_modes.len(), 1);
            assert_eq!(hardware.get_pin(1).unwrap().supported_modes.len(), 2);
        }
    }

    /// Test to decode of "report firmware" command: retrieves the firmware protocol and version.
    #[test]
    fn test_handle_report_firmware() {
        // No firmware name.
        let mut protocol = MockProtocol::default();
        protocol.buf[..5].copy_from_slice(&[0xF0, 0x79, 0x01, 0x0C, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report firmware: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::ReportFirmwareVersion);
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(lock.to_owned().firmware_version, "1.12");
            assert_eq!(lock.to_owned().firmware_name, "Fake protocol");
        }

        // With a custom firmware name.
        let mut protocol = MockProtocol::default();
        protocol.buf[..11].copy_from_slice(&[
            0xF0, 0x79, 0x02, 0x40, 0x66, 0x6F, 0x6F, 0x62, 0x61, 0x72, 0xF7,
        ]);
        let result = protocol.read_and_decode();
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap(), Message::ReportFirmwareVersion);
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(lock.to_owned().firmware_version, "2.64");
            assert_eq!(lock.to_owned().firmware_name, "foobar");
        }

        // Not enough data.
        let mut protocol = MockProtocol::default();
        protocol.buf[..4].copy_from_slice(&[0xF0, 0x79, 0x02, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(result.err().unwrap().to_string(), "Protocol error: Not enough bytes received - 'handle_report_firmware' expected 5 bytes, 4 received.");
    }

    /// Simulate (and test) the handling of a "pin state response" which is reading at a pin value.
    /// Here, we do check that "reading a new value of 30 on pin 3 now in INPUT mode" will be done
    /// properly.
    #[test]
    fn test_handle_pin_state_response() {
        let mut protocol = MockProtocol::default();
        // By default, the value of pin 3 is 3 and mode is OUTPUT:
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(
                lock.to_owned().get_pin(3).unwrap().mode.id,
                PinModeId::OUTPUT
            );
            assert_eq!(lock.to_owned().get_pin(3).unwrap().value, 3);
        }

        // Place the command "value of pin 3 changed to 30": read and handle that
        protocol.buf[..6].copy_from_slice(&[0xF0, 0x6E, 0x03, 0x00, 0x1E, 0xF7]);
        let result = protocol.read_and_decode();

        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap(), Message::PinStateResponse);
        // Now, the value of pin 3 is 30 and mode is INPUT:
        {
            let lock = protocol.get_hardware().read();
            assert_eq!(
                lock.to_owned().get_pin(3).unwrap().mode.id,
                PinModeId::INPUT
            );
            assert_eq!(lock.to_owned().get_pin(3).unwrap().value, 30);
        }

        // Do the same text wil erroneous incoming data:
        let mut protocol = MockProtocol::default();
        protocol.buf[..4].copy_from_slice(&[0xF0, 0x6E, 0x00, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(result.err().unwrap().to_string(), "Protocol error: Not enough bytes received - 'handle_pin_state_response' expected 5 bytes, 4 received.");
    }

    #[test]
    fn test_i2c_config() {
        let mut protocol = MockProtocol::default();
        let result = protocol.i2c_config(100);
        assert!(result.is_ok(), "{:?}", result);
        assert!(
            protocol.buf.starts_with(&[0xF0, 0x78, 0x64, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..5])
        );
    }

    #[test]
    fn test_i2c_read() {
        let mut protocol = MockProtocol::default();
        let result = protocol.i2c_read(0x40, 4);
        assert!(result.is_ok(), "I2C read error: {:?}", result.unwrap_err());
        assert!(
            protocol
                .buf
                .starts_with(&[0xF0, 0x76, 0x40, 0x08, 0x04, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..7])
        );
    }

    #[test]
    fn test_i2c_write() {
        let mut protocol = MockProtocol::default();
        let result = protocol.i2c_write(0x40, &[0x01, 0x02, 0x03]);
        assert!(result.is_ok(), "{:?}", result);
        assert!(
            protocol
                .buf
                .starts_with(&[0xF0, 0x76, 0x40, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&protocol.buf[..11])
        );
    }

    #[test]
    fn test_handle_i2c_reply() {
        // Not enough data.
        let mut protocol = MockProtocol::default();
        protocol.buf[..5].copy_from_slice(&[0xF0, 0x77, 0x02, 0x02, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(result.err().unwrap().to_string(), "Protocol error: Not enough bytes received - 'handle_i2c_reply' expected 9 bytes, 5 received.");

        // Receive an I2C response from i2C address 0x40, register 8, data "coverage".
        let mut protocol = MockProtocol::default();
        protocol.buf[..23].copy_from_slice(&[
            0xF0, 0x77, 0x40, 0x00, 0x08, 0x00, 0x63, 0x00, 0x6F, 0x00, 0x76, 0x00, 0x65, 0x00,
            0x72, 0x00, 0x61, 0x00, 0x67, 0x00, 0x65, 0x00, 0xF7,
        ]);
        let result = protocol.read_and_decode();
        assert!(result.is_ok(), "{:?}", result);
        {
            let lock = protocol.get_hardware().read();
            let hardware = lock.to_owned();
            let data = hardware.i2c_data.get(0).unwrap().clone().data;
            assert_eq!(hardware.i2c_data.len(), 1);
            assert_eq!(hardware.i2c_data[0].address, 64);
            assert_eq!(hardware.i2c_data[0].register, 8);
            assert_eq!(data, vec![0x63, 0x6F, 0x76, 0x65, 0x72, 0x61, 0x67, 0x65]);
            assert_eq!(String::from_utf16(data.as_slice()).unwrap(), "coverage");
        }
    }

    #[test]
    fn test_debug_and_display() {
        let protocol = MockProtocol::default();
        let boxed_protocol: Box<dyn Protocol> = Box::new(MockProtocol::default());
        assert_eq!(protocol.get_protocol_name(), "MockProtocol");
        assert_eq!(protocol.get_protocol_details(), "()");
        assert_eq!(
            format!("{}", boxed_protocol),
            "firmware=Fake protocol, version=fake.2.3, protocol=MockProtocol, connection=\"()\""
        )
    }
}
