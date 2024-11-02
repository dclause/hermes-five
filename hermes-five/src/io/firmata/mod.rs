//! Official Firmata documentation: https://github.com/firmata/protocol
//! Helper unofficial documentation: https://github.com/martin-eden/firmata_protocol/blob/main/protocol.md

pub(crate) mod constants;

use crate::errors::{Error, HardwareError, ProtocolError};
use crate::io::firmata::constants::*;
use crate::io::protocol::IoProtocol;
use crate::io::{I2CReply, IoData, IoTransport, Pin, PinMode, PinModeId, Serial, IO};
use crate::pause;
use crate::utils::task::TaskHandler;
use crate::utils::{task, Range};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

/// Implements the [Firmata protocol](https://github.com/firmata/protocol) within an [`IoProtocol`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct FirmataIo {
    /// Transport layer used to communicate with the device.
    transport: Box<dyn IoTransport>,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(feature = "serde", serde(skip))]
    data: Arc<RwLock<IoData>>,
    /// Inner handler to the polling task.
    #[cfg_attr(feature = "serde", serde(skip))]
    handler: Arc<RwLock<Option<TaskHandler>>>,
}

impl Default for FirmataIo {
    fn default() -> Self {
        Self {
            transport: Box::new(Serial::default()),
            data: Arc::new(Default::default()),
            handler: Arc::new(RwLock::new(None)),
        }
    }
}
impl FirmataIo {
    pub fn new<P: Into<String>>(port: P) -> Self {
        Self {
            transport: Box::new(Serial::new(port)),
            data: Arc::new(Default::default()),
            handler: Arc::new(RwLock::new(None)),
        }
    }
}

impl<T: IoTransport + 'static> From<T> for FirmataIo {
    fn from(transport: T) -> Self {
        Self {
            transport: Box::new(transport),
            data: Arc::new(Default::default()),
            handler: Arc::new(RwLock::new(None)),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoProtocol for FirmataIo {
    #[cfg(not(tarpaulin_include))]
    fn open(&mut self) -> Result<(), Error> {
        self.data.write().connected = false;

        self.transport.open()?;

        // Perform handshake.
        self.handshake()?;

        // Reduce timeout.
        self.transport.set_timeout(Duration::from_millis(500))?;

        self.data.write().connected = true;
        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        self.stop_polling();
        self.data.write().connected = false;
        self.transport.close()?;
        Ok(())
    }

    fn report_analog(&mut self, channel: u8, state: bool) -> Result<(), Error> {
        // trace!"Report analog: {}", state);
        self.transport
            .write(&[REPORT_ANALOG | channel, u8::from(state)])?;
        match state {
            true => {
                self.data.write().analog_reported_channels.push(channel);
                self.start_polling();
            }
            false => {
                let mut lock = self.data.write();
                if let Some(pos) = lock
                    .analog_reported_channels
                    .iter()
                    .position(|&chan| chan == channel)
                {
                    lock.analog_reported_channels.remove(pos);
                    if lock.analog_reported_channels.is_empty() {
                        self.stop_polling();
                    }
                }
            }
        };
        Ok(())
    }

    fn report_digital(&mut self, pin: u8, state: bool) -> Result<(), Error> {
        let port = (pin / 8) as u8;
        let payload = &[REPORT_DIGITAL | port, u8::from(state)];
        // trace!"Report digital: {:02X?}", payload);
        self.transport.write(payload)?;
        match state {
            true => {
                self.data.write().digital_reported_pins.push(pin);
                self.start_polling();
            }
            false => {
                let mut lock = self.data.write();
                if let Some(pos) = lock.digital_reported_pins.iter().position(|&id| id == pin) {
                    lock.digital_reported_pins.remove(pos);
                    if lock.digital_reported_pins.is_empty() {
                        self.stop_polling();
                    }
                }
            }
        };
        Ok(())
    }

    fn sampling_interval(&mut self, interval: u16) -> Result<(), Error> {
        self.transport.write(&[
            START_SYSEX,
            SAMPLING_INTERVAL,
            interval as u8 & SYSEX_REALTIME,
            (interval >> 7) as u8 & SYSEX_REALTIME,
            END_SYSEX,
        ])
    }
}

impl IO for FirmataIo {
    fn get_io(&self) -> &Arc<RwLock<IoData>> {
        &self.data
    }

    fn is_connected(&self) -> bool {
        self.data.read().connected
    }

    fn set_pin_mode(&mut self, pin: u8, mode: PinModeId) -> Result<(), Error> {
        {
            let mut lock = self.data.write();
            let pin_instance = lock.get_pin_mut(pin)?;
            let _mode =
                pin_instance
                    .supports_mode(mode)
                    .ok_or(HardwareError::IncompatibleMode {
                        pin,
                        mode,
                        context: "try to set pin mode",
                    })?;
            pin_instance.mode = _mode;
        }

        self.transport.write(&[SET_PIN_MODE, pin as u8, mode as u8])
    }

    fn digital_write(&mut self, pin: u8, level: bool) -> Result<(), Error> {
        let port = (pin / 8) as u8;
        let mut value: u16 = 0;
        let mut i = 0;

        {
            let mut lock = self.data.write();

            // Check if pin exists
            let pin_instance = lock.get_pin_mut(pin)?;

            // Check if mode is oK.
            pin_instance.validate_current_mode(PinModeId::OUTPUT)?;

            // Store the value we will write to the current pin.
            pin_instance.value = u16::from(level);

            // Loop through all 8 pins of the current "port" to concatenate their value.
            // For instance 01100000 will set to 1 the pin 1 and 2 or current port.
            while i < 8 {
                if lock.get_pin_mut(8 * port + i)?.value != 0 {
                    value |= 1 << i
                }
                i += 1;
            }
        }

        let payload = &[
            DIGITAL_MESSAGE | port,
            value as u8 & SYSEX_REALTIME,
            (value >> 7) as u8 & SYSEX_REALTIME,
        ];
        // trace!"Digital write: {:02X?}", payload);
        self.transport.write(payload)
    }

    fn analog_write(&mut self, pin: u8, level: u16) -> Result<(), Error> {
        // Set the pin value.
        self.data.write().get_pin_mut(pin)?.value = level;

        let payload = if pin > 15 {
            // Extended analog message
            let mut payload = vec![
                START_SYSEX,
                EXTENDED_ANALOG,
                pin as u8,
                level as u8 & SYSEX_REALTIME,
                (level >> 7) as u8 & SYSEX_REALTIME,
            ];
            if level > 0x00004000 {
                payload.push((level >> 14) as u8 & SYSEX_REALTIME);
            }
            payload.push(END_SYSEX);
            payload
        } else {
            // Standard analog message
            vec![
                ANALOG_MESSAGE | pin as u8,
                level as u8 & SYSEX_REALTIME,
                (level >> 7) as u8 & SYSEX_REALTIME,
            ]
        };

        // trace!"Analog write: {:02X?}", payload);
        self.transport.write(&payload)?;
        Ok(())
    }

    fn digital_read(&mut self, _: u8) -> Result<bool, Error> {
        unimplemented!()
    }

    fn analog_read(&mut self, _: u8) -> Result<u16, Error> {
        unimplemented!()
    }

    fn servo_config(&mut self, pin: u8, pwm_range: Range<u16>) -> Result<(), Error> {
        self.transport.write(&[
            START_SYSEX,
            SERVO_CONFIG,
            pin,
            pwm_range.start as u8 & SYSEX_REALTIME,
            (pwm_range.start >> 7) as u8 & SYSEX_REALTIME,
            pwm_range.end as u8 & SYSEX_REALTIME,
            (pwm_range.end >> 7) as u8 & SYSEX_REALTIME,
            END_SYSEX,
        ])
    }

    fn i2c_config(&mut self, delay: u16) -> Result<(), Error> {
        self.transport.write(&[
            START_SYSEX,
            I2C_CONFIG,
            (delay as u8) & SYSEX_REALTIME,
            (delay >> 7) as u8 & SYSEX_REALTIME,
            END_SYSEX,
        ])
    }

    fn i2c_read(&mut self, address: u8, size: u16) -> Result<(), Error> {
        self.transport.write(&[
            START_SYSEX,
            I2C_REQUEST,
            address,
            I2C_READ << 3,
            (size as u8) & SYSEX_REALTIME,
            (size >> 7) as u8 & SYSEX_REALTIME,
            END_SYSEX,
        ])?;
        while self.read_and_decode()? != Message::I2CReply {}
        Ok(())
    }

    fn i2c_write(&mut self, address: u8, data: &[u16]) -> Result<(), Error> {
        let mut buf = vec![START_SYSEX, I2C_REQUEST, address, I2C_WRITE << 3];

        for &i in data.iter() {
            buf.push(i as u8 & SYSEX_REALTIME);
            buf.push((i >> 7) as u8 & SYSEX_REALTIME);
        }

        buf.push(END_SYSEX);

        self.transport.write(&buf)
    }
}

impl FirmataIo {
    /// Sends a software reset request.
    /// <https://github.com/firmata/protocol/blob/master/protocol.md>
    fn software_reset(&mut self) -> Result<(), Error> {
        let payload = &[SYSTEM_RESET];
        // trace!"Software reset: {:02X?}", payload);
        self.transport.write(payload)
    }

    /// Starts a conversation with the board: validate the firmware version and...
    fn handshake(&mut self) -> Result<(), Error> {
        // self.set_connected(false);

        // Forces a software reset: some board do not restart automatically when the connexion is opened.
        // Therefore, running two different software in a raw may result to unexpected settings leftover,
        // for instance the report_analog and report_digital on some pins may continue otherwise.
        self.software_reset()?;

        // The FirmataIo protocol is supposed to send the protocol and firmware version automatically,
        // but it doesn't always do so. The while-loop here ensures that we are now in sync with
        // receiving the expected data. This prevents an initial 'read_and_decode()' call that would
        // otherwise result in a long timeout while waiting to detect the situation
        self.query_firmware()?;
        while self.read_and_decode()? != Message::ReportFirmwareVersion {}

        self.query_capabilities()?;
        while self.read_and_decode()? != Message::CapabilityResponse {}
        self.query_analog_mapping()?;
        while self.read_and_decode()? != Message::AnalogMappingResponse {}

        // println!("PINS {:#?}", self.get_io());
        // self.set_connected(true);
        Ok(())
    }

    /// Query the board for current firmware and protocol information.
    fn query_firmware(&mut self) -> Result<(), Error> {
        let payload = &[START_SYSEX, REPORT_FIRMWARE, END_SYSEX];
        // trace!"Query firmware: {:02X?}", payload);
        self.transport.write(payload)
    }

    /// Query the board for all available capabilities.
    fn query_capabilities(&mut self) -> Result<(), Error> {
        let payload = &[START_SYSEX, CAPABILITY_QUERY, END_SYSEX];
        // trace!"Query capabilities: {:02X?}", payload);
        self.transport.write(payload)
    }

    // ########################################
    // Read/Write on pins

    /// Query the board for available analog pins.
    fn query_analog_mapping(&mut self) -> Result<(), Error> {
        let payload = &[START_SYSEX, ANALOG_MAPPING_QUERY, END_SYSEX];
        // trace!"Query analog mapping: {:02X?}", payload);
        self.transport.write(payload)
    }

    // ########################################
    // FirmataIo read & handle functions

    /// Read from the protocol, parse and return its type.
    /// The following method should use Firmata protocol such as defined here:
    /// <https://github.com/firmata/protocol/blob/master/protocol.md>
    fn read_and_decode(&mut self) -> Result<Message, Error> {
        let mut buf = vec![0; 3];
        self.transport.read_exact(&mut buf)?;

        match buf[0] {
            REPORT_PROTOCOL_VERSION => self.handle_protocol_version(&buf),
            ANALOG_MESSAGE..=ANALOG_MESSAGE_BOUND => self.handle_analog_message(&buf),
            DIGITAL_MESSAGE..=DIGITAL_MESSAGE_BOUND => self.handle_digital_message(&buf),
            START_SYSEX => self.handle_sysex_message(&mut buf),
            _ => {
                // trace!"IoPlugin: unexpected data: {:02X?}", buf.as_slice());
                Ok(Message::EmptyResponse)
            }
        }
    }

    /// Handle a REPORT_VERSION_RESPONSE message (0xF9 - return the firmware version).
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#message-types>
    fn handle_protocol_version(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut lock = self.get_io().write();
        lock.protocol_version = format!("{}.{}", buf[1], buf[2]);
        // trace!"Received protocol version: {}", lock.protocol_version);
        Ok(Message::ReportProtocolVersion)
    }

    /// Handle an ANALOG_MESSAGE message (0xE0 - report state of an analog pin)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion>
    fn handle_analog_message(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let pin = (buf[0] & 0x0F) + 14;
        let value = (buf[1] as u16) | ((buf[2] as u16) << 7);
        // trace!"Received analog message: pin({})={}", pin, value);
        self.get_io().write().get_pin_mut(pin)?.value = value;
        Ok(Message::Analog)
    }

    /// Handle a DIGITAL_MESSAGE message (0x90 - report state of a digital pin/port)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion>
    fn handle_digital_message(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let port = buf[0] & 0x0F;
        let value = (buf[1] as u16) | ((buf[2] as u16) << 7);
        // trace!"Received digital message: pin({})={}", port, value);

        for i in 0..8 {
            let pin = (8 * port) + i;
            let mode: PinModeId = self.get_io().read().get_pin(pin)?.mode.id;
            if mode == PinModeId::INPUT || mode == PinModeId::PULLUP {
                self.get_io().write().get_pin_mut(pin)?.value = (value >> (i & 0x07)) & 0x01;
            }
        }
        Ok(Message::Digital)
    }

    /// Handle a START_SYSEX message: dispatch to various message/command/response using the sysex format.
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#sysex-message-format>
    fn handle_sysex_message(&mut self, buf: &mut Vec<u8>) -> Result<Message, Error> {
        if buf[1] == END_SYSEX || buf[2] == END_SYSEX {
            return Ok(Message::EmptyResponse);
        }

        loop {
            // Read until END_SYSEX.
            let mut byte = [0];
            self.transport.read_exact(&mut byte)?;
            buf.push(byte[0]);
            if byte[0] == END_SYSEX {
                break;
            }
        }
        match buf[1] {
            ANALOG_MAPPING_RESPONSE => self.handle_analog_mapping_response(buf),
            CAPABILITY_RESPONSE => self.handle_capability_response(buf),
            REPORT_FIRMWARE => self.handle_firmware_report(buf),
            I2C_REPLY => self.handle_i2c_reply(buf),
            PIN_STATE_RESPONSE => self.handle_pin_state_response(buf),
            _ => {
                // trace!"Sysex: unexpected data: {:02X?}", buf.as_slice());
                Ok(Message::EmptyResponse)
            }
        }
    }

    /// Handle an ANALOG_MAPPING_RESPONSE message (0x6A - reply with analog pins mapping info).
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#analog-mapping-query>
    fn handle_analog_mapping_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut lock = self.get_io().write();
        let mut i = 2;
        while buf[i] != END_SYSEX {
            if buf[i] != SYSEX_REALTIME {
                let pin = &mut lock.get_pin_mut((i - 2) as u8)?;
                pin.mode = pin.supports_mode(PinModeId::ANALOG).ok_or(
                    HardwareError::IncompatibleMode {
                        pin: (i - 2) as u8,
                        mode: PinModeId::ANALOG,
                        context: "handle_analog_mapping_response",
                    },
                )?;
                pin.name = format!("A{}", buf[i]);
                pin.channel = Some(buf[i]);
            }
            i += 1;
        }
        Ok(Message::AnalogMappingResponse)
    }

    /// Handle a CAPABILITY_RESPONSE message (0x6C - reply with supported modes and resolution)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#capability-query>
    fn handle_capability_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let mut id = 0;
        let mut i = 2;
        let mut lock = self.get_io().write();
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
                name: format!("D{}", id),
                supported_modes,
                ..Default::default()
            };
            if !pin.supported_modes.is_empty() {
                pin.mode = *pin.supported_modes.first().unwrap();
            }
            lock.pins.insert(pin.id, pin);

            i += 1;
            id += 1;
        }

        // trace!"Received capability response: @see hardware.pins");
        Ok(Message::CapabilityResponse)
    }

    /// Handle a REPORT_FIRMWARE message (0x79 - report name and version of the firmware).
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#query-firmware-name-and-version>
    fn handle_firmware_report(&mut self, buf: &[u8]) -> Result<Message, Error> {
        if buf.len() < 5 {
            return Err(Error::from(ProtocolError::MessageTooShort {
                operation: "handle_firmware_report",
                expected: 5,
                received: buf.len(),
            }));
        }
        let major = buf[2];
        let minor = buf[3];
        let mut lock = self.get_io().write();
        lock.firmware_version = format!("{}.{}", major, minor);
        // trace!"Received firmware version: {}", lock.firmware_version);
        if buf.len() > 5 {
            lock.firmware_name = std::str::from_utf8(&buf[4..buf.len() - 1])?
                .to_string()
                .replace('\0', "");
            // trace!"Received firmware name: {}", lock.firmware_name);
        }
        Ok(Message::ReportFirmwareVersion)
    }

    /// Handle an I2C_REPLY message (0x6E - read and decode an i2c message)
    /// <https://github.com/firmata/protocol/blob/master/i2c.md>
    fn handle_i2c_reply(&mut self, buf: &[u8]) -> Result<Message, Error> {
        // trace!"I2C REPLY: {}", format_as_hex(buf));

        if buf.len() < 8 {
            return Err(Error::from(ProtocolError::MessageTooShort {
                operation: "handle_i2c_reply",
                expected: 9,
                received: buf.len(),
            }));
        }
        let mut reply = I2CReply {
            address: buf[2],
            register: buf[4] | (buf[5] << 7),
            data: vec![buf[6] | (buf[7] << 7)],
        };
        let mut i = 8;
        while buf[i] != END_SYSEX {
            reply.data.push((buf[i]) | (buf[i + 1] << 7));
            i += 2;
        }
        self.get_io().write().i2c_data.push(reply);
        Ok(Message::I2CReply)
    }

    /// Handle a PIN_STATE_RESPONSE message (0x6E - report pin current mode and state)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#pin-state-query>
    fn handle_pin_state_response(&mut self, buf: &[u8]) -> Result<Message, Error> {
        let pin = buf[2];
        if buf.len() < 4 || buf[3] == END_SYSEX {
            return Err(Error::from(ProtocolError::MessageTooShort {
                operation: "handle_pin_state_response",
                expected: 5,
                received: buf.len(),
            }));
        }

        let mut lock = self.get_io().write();
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
        // trace!"Received pin state: {:?}", pin);
        Ok(Message::PinStateResponse)
    }

    /// Manually attaches the board value change listener. This is only used for input events.
    /// This should never be needed unless you manually `detach()` the sensor first for some reason
    /// and want it to start being reactive to events again.
    pub fn start_polling(&self) {
        if self.handler.read().is_none() {
            let mut self_clone = self.clone();
            *self.handler.write() = Some(
                task::run(async move {
                    // Infinite loop to listen for inputs from the board.
                    // @todo this is constant polling. Evaluate if this is the right solution and the polling resolution.
                    loop {
                        let _ = self_clone.read_and_decode();
                        pause!(1);
                    }

                    #[allow(unreachable_code)]
                    Ok(())
                })
                .unwrap(),
            );
        }
    }

    /// Detaches the interval associated with the button.
    /// This means the button won't react anymore to value changes.
    pub fn stop_polling(&self) {
        if let Some(handler) = self.handler.read().as_ref() {
            handler.abort();
        }
        *self.handler.write() = None;
    }
}

impl Display for FirmataIo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let data = self.data.read();
        write!(
            f,
            "{} [firmware={}, version={}, protocol={}, transport={}]",
            self.get_name(),
            data.firmware_name,
            data.firmware_version,
            data.protocol_version,
            self.transport
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::io::constants::Message;
    use crate::io::protocol::IoProtocol;
    use crate::io::{FirmataIo, PinModeId, Serial, IO};
    use crate::mocks::create_test_plugin_io_data;
    use crate::utils::{format_as_hex, Range};
    use hermes_five::mocks::transport_layer::MockTransportLayer;
    use parking_lot::lock_api::RwLock;
    use std::sync::Arc;

    fn _create_mock_protocol() -> FirmataIo {
        let mut protocol = FirmataIo::from(MockTransportLayer::default());
        protocol.data = Arc::new(RwLock::new(create_test_plugin_io_data()));
        protocol
    }

    fn _create_mock_protocol_with_data(data: &[u8]) -> FirmataIo {
        let mut transport = MockTransportLayer::default();
        transport.read_buf[..data.len()].copy_from_slice(data);
        let mut protocol = FirmataIo::from(transport);
        protocol.data = Arc::new(RwLock::new(create_test_plugin_io_data()));
        protocol
    }

    fn _get_mock_transport(protocol: &FirmataIo) -> &MockTransportLayer {
        protocol
            .transport
            .as_any()
            .downcast_ref::<MockTransportLayer>()
            .unwrap()
    }

    #[test]
    fn test_creation() {
        let protocol = FirmataIo::default();
        let transport = protocol.transport.as_any().downcast_ref::<Serial>();
        assert!(transport.is_some());

        let protocol = FirmataIo::new("try");
        let transport = protocol.transport.as_any().downcast_ref::<Serial>();
        assert!(transport.is_some());
        assert_eq!(transport.unwrap().get_port(), String::from("try"));

        let protocol = FirmataIo::from(MockTransportLayer::default());
        let transport = protocol
            .transport
            .as_any()
            .downcast_ref::<MockTransportLayer>();
        assert!(transport.is_some());
    }

    #[test]
    fn test_software_reset() {
        let mut protocol = _create_mock_protocol();

        let result = protocol.software_reset();
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.starts_with(&[0xFF]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..1])
        );
    }

    #[test]
    fn test_handshake() {
        let mut transport = _create_mock_protocol_with_data(&[
            0xF0, 0x79, 0x01, 0x0C, 0xF7, // Result for query firmware
            0xF0, 0x6C, 0x00, 0x08, 0x7F, 0x00, 0x08, 0x01, 0x08, 0x7F,
            0xF7, // Result for report capabilities
            0xF0, 0x6A, 0x7F, 0x7F, 0x7F, 0xF7, // Result for report capabilities
        ]);
        let result = transport.handshake();
        assert!(result.is_ok(), "{:?}", result);
        let transport = _get_mock_transport(&transport);
        assert!(
            transport.write_buf.starts_with(&[
                0xFF, // software reset
                0xF0, 0x79, 0xF7, // query firmware
                0xF0, 0x6B, 0xF7, // query capacities
                0xF0, 0x69, 0xF7, // query analog mapping
            ]),
            "Sending sequence is correct"
        )
    }

    #[test]
    fn test_open() {
        let mut transport = _create_mock_protocol_with_data(&[
            0xF0, 0x79, 0x01, 0x0C, 0xF7, // Result for query firmware
            0xF0, 0x6C, 0x00, 0x08, 0x7F, 0x00, 0x08, 0x01, 0x08, 0x7F,
            0xF7, // Result for report capabilities
            0xF0, 0x6A, 0x7F, 0x7F, 0x7F, 0xF7, // Result for report capabilities
        ]);
        let result = transport.open();
        assert!(result.is_ok(), "{:?}", result);
        assert!(transport.is_connected())
    }

    #[test]
    fn test_simple_analog_write() {
        let mut protocol = _create_mock_protocol();
        let result = protocol.analog_write(0, 170);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.starts_with(&[0xE0, 0x2A, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..3])
        );
        {
            let lock = protocol.get_io().read();
            let pin = lock.get_pin(0).unwrap();
            assert_eq!(pin.value, 170, "Pin value updated");
        }

        let result = protocol.analog_write(66, 0);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Unknown pin 66."
        );
    }

    #[test]
    fn test_extended_analog_write() {
        let mut protocol = _create_mock_protocol();
        // Note1: the pin to use is over 15, so we use extended protocol.
        // Note2: the value sent is over 16384 (0x00004000) so we use multibyte sending.
        let result = protocol.analog_write(22, 17000);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .starts_with(&[0xF0, 0x6F, 0x16, 0x68, 0x04, 0x01, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..7])
        );
        {
            let lock = protocol.get_io().read();
            let pin = lock.get_pin(22).unwrap();
            assert_eq!(pin.value, 17000, "Pin value updated");
        }

        let result = protocol.analog_write(42, 0);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Unknown pin 42."
        );
    }

    #[test]
    fn test_digital_write() {
        let mut protocol = _create_mock_protocol();

        // TEST
        let result = protocol.digital_write(13, true);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.starts_with(&[0x91, 0x7F, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..3])
        );

        {
            let lock = protocol.get_io().read();
            let pin = lock.get_pin(13).unwrap();
            assert_eq!(pin.value, 1);
            let pin = lock.get_pin(11).unwrap();
            assert_eq!(pin.value, 11, "Other pin value does not change");
        }

        let result = protocol.digital_write(66, true);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Unknown pin 66."
        );
    }

    #[test]
    fn test_set_pin_mode() {
        let mut protocol = _create_mock_protocol();

        {
            let lock = protocol.get_io().read();
            let pin = lock.get_pin(8).unwrap();
            assert_eq!(pin.mode.id, PinModeId::PWM);
        }

        let result = protocol.set_pin_mode(8, PinModeId::OUTPUT);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.starts_with(&[0xF4, 0x08, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..3])
        );

        {
            let lock = protocol.get_io().read();
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
        let mut protocol = _create_mock_protocol();

        let result = protocol.servo_config(8, Range::from([500, 2500]));
        assert!(
            result.is_ok(),
            "Servo config error: {:?}",
            result.unwrap_err()
        );

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .starts_with(&[0xF0, 0x70, 0x08, 0x74, 0x03, 0x44, 0x13, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..8])
        );
    }

    #[test]
    fn test_query_firmware() {
        let mut protocol = _create_mock_protocol();

        let result = protocol.query_firmware();
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.starts_with(&[0xF0, 0x79, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..3])
        );
    }

    #[test]
    fn test_query_capabilities() {
        let mut protocol = _create_mock_protocol();

        let result = protocol.query_capabilities();
        assert!(
            result.is_ok(),
            "Query capabilities: {:?}",
            result.unwrap_err()
        );

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.starts_with(&[0xF0, 0x6B, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..3])
        );
    }

    #[test]
    fn test_query_analog_mapping() {
        let mut protocol = _create_mock_protocol();

        let result = protocol.query_analog_mapping();
        assert!(
            result.is_ok(),
            "Query analog mapping: {:?}",
            result.unwrap_err()
        );

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.starts_with(&[0xF0, 0x69, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..3])
        );
    }

    #[test]
    fn test_sampling_interval() {
        let mut protocol = _create_mock_protocol();

        let result = protocol.sampling_interval(100);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .starts_with(&[0xF0, 0x7A, 0x64, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..5])
        );
    }

    #[hermes_macros::test]
    fn test_report_analog() {
        let mut protocol = _create_mock_protocol();
        assert!(protocol.data.read().analog_reported_channels.is_empty());

        // Check data sent when enable reporting
        let result = protocol.report_analog(2, true);
        assert!(result.is_ok(), "{:?}", result);
        let _ = protocol.report_analog(3, true);
        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.starts_with(&[0xC2, 0x01, 0xC3, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..4])
        );

        // Reporting enables a watch task.
        assert!(protocol.handler.read().is_some());
        assert_eq!(protocol.data.read().analog_reported_channels.len(), 2);
        assert!(protocol.data.read().analog_reported_channels.contains(&2));
        assert!(protocol.data.read().analog_reported_channels.contains(&3));

        // Remove a report analog keeps the watch task.
        let _ = protocol.report_analog(2, false);
        assert_eq!(protocol.data.read().analog_reported_channels.len(), 1);
        assert!(!protocol.data.read().analog_reported_channels.contains(&2));
        assert!(protocol.data.read().analog_reported_channels.contains(&3));
        assert!(protocol.handler.read().is_some());

        // Remove last report analog kills the watch task.
        let _ = protocol.report_analog(3, false);
        assert!(protocol.data.read().analog_reported_channels.is_empty());
        assert!(protocol.handler.read().is_none());
    }

    #[hermes_macros::test]
    fn test_report_digital() {
        let mut protocol = _create_mock_protocol();
        assert!(protocol.data.read().digital_reported_pins.is_empty());

        // Check data sent when enable reporting
        let result = protocol.report_digital(1, true);
        assert!(result.is_ok(), "{:?}", result);
        let result = protocol.report_digital(13, true);
        assert!(result.is_ok(), "{:?}", result);
        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.starts_with(&[0xD0, 0x01, 0xD1, 0x01]), // 0xD0 for port 0 (pin 1-7); 0xD1 for port 1 (pin 8-15)
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..4])
        );

        // Reporting enables a watch task.
        assert!(protocol.handler.read().is_some());
        assert_eq!(protocol.data.read().digital_reported_pins.len(), 2);
        assert!(protocol.data.read().digital_reported_pins.contains(&1));
        assert!(protocol.data.read().digital_reported_pins.contains(&13));

        // Remove a report analog keeps the watch task.
        let _ = protocol.report_digital(1, false);
        assert_eq!(protocol.data.read().digital_reported_pins.len(), 1);
        assert!(!protocol.data.read().digital_reported_pins.contains(&1));
        assert!(protocol.data.read().digital_reported_pins.contains(&13));
        assert!(protocol.handler.read().is_some());

        // Remove last report analog kills the watch task.
        let _ = protocol.report_digital(13, false);
        assert!(protocol.data.read().digital_reported_pins.is_empty());
        assert!(protocol.handler.read().is_none());
    }

    #[test]
    fn test_handle_protocol_version() {
        let mut protocol = _create_mock_protocol_with_data(&[0xF9, 0x01, 0x19]);

        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report version: {:?}",
            result.unwrap_err()
        );

        assert_eq!(result.unwrap(), Message::ReportProtocolVersion);
        {
            let lock = protocol.get_io().read();
            assert_eq!(lock.to_owned().protocol_version, "1.25");
        }
    }

    #[test]
    fn test_handle_analog_message() {
        let mut transport = _create_mock_protocol_with_data(&[0xE1, 0xDE, 0x00]);

        let result = transport.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report version: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::Analog);
        {
            let lock = transport.get_io().read();
            assert_eq!(lock.to_owned().get_pin(15).unwrap().value, 222);
        }
    }

    #[test]
    fn test_handle_digital_message() {
        let mut protocol = _create_mock_protocol_with_data(&[0x91, 0x00, 0x00]);
        {
            let lock = protocol.get_io().read();
            assert_eq!(lock.to_owned().get_pin(10).unwrap().value, 10);
            assert_eq!(lock.to_owned().get_pin(12).unwrap().value, 12);
        }

        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report version: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::Digital);
        {
            let lock = protocol.get_io().read();
            assert_eq!(lock.to_owned().get_pin(10).unwrap().value, 0);
            assert_eq!(lock.to_owned().get_pin(12).unwrap().value, 12);
        }
    }

    #[test]
    fn test_handle_empty_sysex() {
        // Unexpected data when the first byte received in not a valid command.
        let mut protocol = _create_mock_protocol_with_data(&[0x11]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::EmptyResponse);

        // Unexpected data when the first byte is a sysex, the size is plausible,
        // but the second is not a valid sysex command.
        let mut protocol = _create_mock_protocol_with_data(&[0xF0, 0x11, 0x11, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::EmptyResponse);

        // Empty command error when a sysex is received and closed immediately.
        let mut protocol = _create_mock_protocol_with_data(&[0xF0, 0xF7]);
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
        let mut protocol = _create_mock_protocol_with_data(&[0xF0, 0x6A, 0x01, 0x7F, 0x7F, 0xF7]);
        {
            let lock = protocol.get_io().read();
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
            let lock = protocol.get_io().read();
            assert_eq!(lock.to_owned().get_pin(0).unwrap().channel, Some(1));
        }

        // Unsupported possible data
        let mut protocol = _create_mock_protocol_with_data(&[0xF0, 0x6A, 0x01, 0x01, 0x01, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Pin (2) not compatible with mode (ANALOG) - handle_analog_mapping_response."
        );
    }

    #[test]
    fn test_handle_capability_response() {
        let mut protocol = _create_mock_protocol_with_data(&[
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
            let lock = protocol.get_io().read();
            let hardware = lock.to_owned();
            assert_eq!(hardware.pins.len(), 2, "{:?}", hardware.pins);
            assert_eq!(hardware.get_pin(0).unwrap().supported_modes.len(), 1);
            assert_eq!(hardware.get_pin(1).unwrap().supported_modes.len(), 2);
        }
    }

    /// Test to decode of "report firmware" command: retrieves the firmware protocol and version.
    #[test]
    fn test_handle_firmware_report() {
        // No firmware name.
        let mut protocol = _create_mock_protocol_with_data(&[0xF0, 0x79, 0x01, 0x0C, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report firmware: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::ReportFirmwareVersion);
        {
            let lock = protocol.get_io().read();
            assert_eq!(lock.to_owned().firmware_version, "1.12");
            assert_eq!(lock.to_owned().firmware_name, "Fake protocol");
        }

        // With a custom firmware name.
        let mut protocol = _create_mock_protocol_with_data(&[
            0xF0, 0x79, 0x02, 0x40, 0x66, 0x6F, 0x6F, 0x62, 0x61, 0x72, 0xF7,
        ]);
        let result = protocol.read_and_decode();
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap(), Message::ReportFirmwareVersion);
        {
            let lock = protocol.get_io().read();
            assert_eq!(lock.to_owned().firmware_version, "2.64");
            assert_eq!(lock.to_owned().firmware_name, "foobar");
        }

        // Not enough data.
        let mut protocol = _create_mock_protocol_with_data(&[0xF0, 0x79, 0x02, 0xF7]);
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(result.err().unwrap().to_string(), "Protocol error: Not enough bytes received - 'handle_firmware_report' expected 5 bytes, 4 received.");
    }

    /// Simulate (and test) the handling of a "pin state response" which is reading at a pin value.
    /// Here, we do check that "reading a new value of 30 on pin 3 now in INPUT mode" will be done
    /// properly.
    #[test]
    fn test_handle_pin_state_response() {
        let mut protocol = _create_mock_protocol_with_data(&[
            0xF0, 0x6E, 0x03, 0x00, 0x1E, 0xF7, 0xF0, 0x6E, 0x00, 0xF7,
        ]);
        // By default, the value of pin 3 is 3 and mode is OUTPUT:
        {
            let lock = protocol.get_io().read();
            assert_eq!(
                lock.to_owned().get_pin(3).unwrap().mode.id,
                PinModeId::OUTPUT
            );
            assert_eq!(lock.to_owned().get_pin(3).unwrap().value, 3);
        }

        // Place the command "value of pin 3 changed to 30": read and handle that
        let result = protocol.read_and_decode();
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap(), Message::PinStateResponse);
        // Now, the value of pin 3 is 30 and mode is INPUT:
        {
            let lock = protocol.get_io().read();
            assert_eq!(
                lock.to_owned().get_pin(3).unwrap().mode.id,
                PinModeId::INPUT
            );
            assert_eq!(lock.to_owned().get_pin(3).unwrap().value, 30);
        }

        // Do the same text wil erroneous incoming data:
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(result.err().unwrap().to_string(), "Protocol error: Not enough bytes received - 'handle_pin_state_response' expected 5 bytes, 4 received.");
    }

    #[test]
    fn test_i2c_config() {
        let mut protocol = _create_mock_protocol();

        let result = protocol.i2c_config(100);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .starts_with(&[0xF0, 0x78, 0x64, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..5])
        );
    }

    #[test]
    fn test_i2c_read() {
        let mut protocol = _create_mock_protocol_with_data(&[
            0xF0, 0x77, 0x40, 0x00, 0x42, 0x42, 0x42, 0x42, 0xF7, // mock 4 bytes i2c answer.
        ]);

        let result = protocol.i2c_read(0x40, 4); // wait and read 4 bytes.
        assert!(result.is_ok(), "I2C read error: {:?}", result.unwrap_err());

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .starts_with(&[0xF0, 0x76, 0x40, 0x08, 0x04, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..7])
        );
    }

    #[test]
    fn test_i2c_write() {
        let mut protocol = _create_mock_protocol();

        let result = protocol.i2c_write(0x40, &[0x01, 0x02, 0x03]);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .starts_with(&[0xF0, 0x76, 0x40, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf[..11])
        );
    }

    #[test]
    fn test_handle_i2c_reply() {
        // Not enough data.
        let mut protocol = _create_mock_protocol_with_data(&[0xF0, 0x77, 0x02, 0x02, 0xF7]);

        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(result.err().unwrap().to_string(), "Protocol error: Not enough bytes received - 'handle_i2c_reply' expected 9 bytes, 5 received.");

        // Receive an I2C response from i2C address 0x40, register 8, data "coverage".
        let mut protocol = _create_mock_protocol_with_data(&[
            0xF0, 0x77, 0x40, 0x00, 0x08, 0x00, 0x63, 0x00, 0x6F, 0x00, 0x76, 0x00, 0x65, 0x00,
            0x72, 0x00, 0x61, 0x00, 0x67, 0x00, 0x65, 0x00, 0xF7,
        ]);
        let result = protocol.read_and_decode();
        assert!(result.is_ok(), "{:?}", result);
        {
            let data = protocol.get_io().read();
            assert_eq!(data.i2c_data.len(), 1);
            assert_eq!(data.i2c_data[0].address, 64);
            assert_eq!(data.i2c_data[0].register, 8);
            let data = data.i2c_data[0].clone().data;
            assert_eq!(data, vec![0x63, 0x6F, 0x76, 0x65, 0x72, 0x61, 0x67, 0x65]);
            assert_eq!(String::from_utf8_lossy(data.as_slice()), "coverage");
        }
    }

    #[test]
    fn test_debug_and_display() {
        let protocol = _create_mock_protocol();
        let boxed_protocol: Box<dyn IoProtocol> = Box::new(protocol);
        // assert_eq!(protocol.get_protocol_name(), "MockIoProtocol");
        assert_eq!(
            format!("{}", boxed_protocol),
            "FirmataIo [firmware=Fake protocol, version=fake.2.3, protocol=fake.1.0, transport=MockTransportLayer]"
        )
    }
}
