//! RemoIo is an `IoProtocol` to remotely control a `Board` via a given `IoTransport`.
//!
//! This IoProtocol uses Firmata protocol to normalize communications through the IoTransport layer.
//! Official Firmata documentation: https://github.com/firmata/protocol
//! Helper unofficial documentation: https://github.com/martin-eden/firmata_protocol/blob/main/protocol.md

use crate::errors::{Error, ProtocolError};
use crate::hardware::{I2CReply, LowLevelApi, LowLevelApiExt, Pin, PinMode, PinModeId};
use crate::pause;
use crate::protocols::constants::*;
use crate::protocols::IoProtocol;
use crate::transports::{IoTransport, Serial};
use crate::utils::task::TaskHandler;
use crate::utils::{task, ArcOnceLockExt, Range};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

/// `RemoteIo` is the protocol used to control boards and devices remotely using various compatible `IoProtocol`.
/// Have a look at the [examples/io folder](https://github.com/dclause/hermes-five/tree/develop/hermes-five/examples/io) for examples.
///
/// Internally, implements the [Firmata protocol](https://github.com/firmata/protocol) within an [`IoProtocol`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct RemoteIo {
    /// Transport layer used to communicate with the device.
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::serde_arc_transport"))]
    transport: Arc<dyn IoTransport>,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol_version: Arc<OnceLock<String>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    firmware_name: Arc<OnceLock<String>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    firmware_version: Arc<OnceLock<String>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pins: Arc<OnceLock<HashMap<u8, Arc<Pin>>>>,
    /// A bitmask tracking which analog pins are enabled for reporting.
    /// Each bit corresponds to a `Pin.id`; if bit `n` is set, analog pin `n` is reported.
    /// Uses `AtomicUsize` for efficient, thread-safe updates.
    #[cfg_attr(feature = "serde", serde(skip))]
    analog_reported_channels: Arc<AtomicUsize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    digital_reported_pins: Arc<AtomicUsize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    i2c_data: Arc<RwLock<Vec<I2CReply>>>,

    /// Inner handler to the polling task.
    #[cfg_attr(feature = "serde", serde(skip))]
    handler: Arc<RwLock<Option<TaskHandler>>>,
}

impl Default for RemoteIo {
    fn default() -> Self {
        Self {
            transport: Arc::new(Serial::default()),
            protocol_version: Default::default(),
            firmware_name: Default::default(),
            firmware_version: Default::default(),
            pins: Default::default(),
            analog_reported_channels: Default::default(),
            digital_reported_pins: Default::default(),
            i2c_data: Default::default(),
            handler: Default::default(),
        }
    }
}

impl RemoteIo {
    pub fn new<P: Into<String>>(port: P) -> Self {
        Self {
            transport: Arc::new(Serial::new(port)),
            ..Default::default()
        }
    }
    pub fn get_transport(&self) -> Arc<dyn IoTransport> {
        self.transport.clone()
    }

    /// Sets the reporting status for the given digital pin.
    fn set_digital_reporting(&self, pin: u8, report: bool) {
        if report {
            self.digital_reported_pins
                .fetch_or(1 << pin, Ordering::Relaxed);
        } else {
            self.digital_reported_pins
                .fetch_and(!(1 << pin), Ordering::Relaxed);
        };
    }

    /// Gets the reporting status for the given digital pin.
    fn is_digital_reporting(&self, pin: u8) -> bool {
        self.digital_reported_pins.load(Ordering::Relaxed) & (1 << pin) != 0
    }

    /// Sets the reporting status for the given analog pin.
    fn set_analog_reporting(&self, channel: u8, report: bool) {
        if report {
            self.analog_reported_channels
                .fetch_or(1 << channel, Ordering::Relaxed);
        } else {
            self.analog_reported_channels
                .fetch_and(!(1 << channel), Ordering::Relaxed);
        };
    }

    /// Gets the reporting status for the given analog pin.
    fn is_analog_reporting(&self, channel: u8) -> bool {
        self.analog_reported_channels.load(Ordering::Relaxed) & (1 << channel) != 0
    }
}

impl<T: IoTransport + 'static> From<T> for RemoteIo {
    fn from(transport: T) -> Self {
        Self {
            transport: Arc::new(transport),
            ..Default::default()
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoProtocol for RemoteIo {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn open(&self) -> Result<(), Error> {
        self.transport.open()?;

        // Perform handshake.
        self.handshake()?;

        Ok(())
    }

    fn close(&self) -> Result<(), Error> {
        self.stop_polling();
        self.transport.close()?;
        Ok(())
    }

    fn report_analog(&self, channel: u8, state: bool) -> Result<(), Error> {
        // trace!("Report analog: {}", state);
        self.transport
            .write(&[REPORT_ANALOG | channel, u8::from(state)])?;

        match state {
            true => {
                self.set_analog_reporting(channel, true);
                self.start_polling();
            }
            false => {
                self.set_analog_reporting(channel, false);
                if self.analog_reported_channels.load(Ordering::Relaxed) == 0
                    && self.digital_reported_pins.load(Ordering::Relaxed) == 0
                {
                    self.stop_polling();
                }
            }
        };
        Ok(())
    }

    fn report_digital(&self, pin: u8, state: bool) -> Result<(), Error> {
        let port = pin / 8;
        let payload = &[REPORT_DIGITAL | port, u8::from(state)];
        // trace!("Report digital: {:02X?}", payload);
        self.transport.write(payload)?;
        match state {
            true => {
                self.set_digital_reporting(pin, true);
                self.start_polling();
            }
            false => {
                self.set_digital_reporting(pin, false);
                if self.analog_reported_channels.load(Ordering::Relaxed) == 0
                    && self.digital_reported_pins.load(Ordering::Relaxed) == 0
                {
                    self.stop_polling();
                }
            }
        };
        Ok(())
    }

    fn sampling_interval(&self, interval: u16) -> Result<(), Error> {
        self.transport.write(&[
            START_SYSEX,
            SAMPLING_INTERVAL,
            interval as u8 & SYSEX_REALTIME,
            (interval >> 7) as u8 & SYSEX_REALTIME,
            END_SYSEX,
        ])
    }
}

impl LowLevelApi for RemoteIo {
    #[inline(always)]
    fn get_protocol_name(&self) -> &str {
        "RemoteIo"
    }

    fn get_protocol_version(&self) -> &str {
        self.protocol_version
            .get()
            .map(|s| s.as_str())
            .unwrap_or("N/A")
    }

    fn get_firmware_name(&self) -> &str {
        self.firmware_name
            .get()
            .map(|s| s.as_str())
            .unwrap_or("N/A")
    }

    fn get_firmware_version(&self) -> &str {
        self.firmware_version
            .get()
            .map(|s| s.as_str())
            .unwrap_or("N/A")
    }

    #[inline(always)]
    fn get_pins(&self) -> &HashMap<u8, Arc<Pin>> {
        self.pins.get().unwrap()
    }

    fn set_pin_mode(&self, pin: u8, mode: PinModeId) -> Result<(), Error> {
        self.get_pin(pin)?.set_pin_mode(mode)?;
        self.transport.write(&[SET_PIN_MODE, pin, mode as u8])
    }

    fn digital_write(&self, pin: u8, level: bool) -> Result<(), Error> {
        let port = pin / 8;
        let mut value: u16 = 0;
        let mut i = 0;

        // Check if pin exists
        let pin_instance = self.get_pin(pin)?;

        // Check if mode is oK.
        pin_instance.ensure_mode_is(PinModeId::OUTPUT)?;

        // Store the value we will write to the current pin.
        pin_instance.set_value(if level { u16::MAX } else { u16::MIN });

        // Loop through all 8 pins of the current "port" to concatenate their value.
        // For instance 01100000 will set to 1 the pin 1 and 2 or current port.
        while i < 8 {
            if self.get_pin_value(8 * port + i)? != 0 {
                value |= 1 << i
            }
            i += 1;
        }

        let payload = &[
            DIGITAL_MESSAGE | port,
            value as u8 & SYSEX_REALTIME,
            (value >> 7) as u8 & SYSEX_REALTIME,
        ];
        // trace!("Digital write: {:02X?}", payload);
        self.transport.write(payload)
    }

    fn analog_write(&self, pin: u8, level: u16) -> Result<(), Error> {
        // Set the pin value.
        self.get_pin(pin)?.set_value(level);

        let payload = if pin > 15 {
            // Extended analog message
            let mut payload = vec![
                START_SYSEX,
                EXTENDED_ANALOG,
                pin,
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
                ANALOG_MESSAGE | pin,
                level as u8 & SYSEX_REALTIME,
                (level >> 7) as u8 & SYSEX_REALTIME,
            ]
        };

        // trace!("Analog write: {:02X?}", payload);
        self.transport.write(&payload)?;
        Ok(())
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn digital_read(&self, _: u8) -> Result<bool, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn analog_read(&self, _: u8) -> Result<u16, Error> {
        Err(Error::NotImplemented)
    }

    fn servo_config(&self, pin: u8, pwm_range: Range<u16>) -> Result<(), Error> {
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

    fn i2c_config(&self, delay: u16) -> Result<(), Error> {
        self.transport.write(&[
            START_SYSEX,
            I2C_CONFIG,
            (delay as u8) & SYSEX_REALTIME,
            (delay >> 7) as u8 & SYSEX_REALTIME,
            END_SYSEX,
        ])
    }

    fn i2c_read(&self, address: u8, size: u16) -> Result<(), Error> {
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

    fn i2c_write(&self, address: u8, data: &[u16]) -> Result<(), Error> {
        let mut buf = vec![START_SYSEX, I2C_REQUEST, address, I2C_WRITE << 3];

        for &i in data.iter() {
            buf.push(i as u8 & SYSEX_REALTIME); // bits 0-6
            buf.push((i >> 7) as u8 & SYSEX_REALTIME); // bits 7-13
        }

        buf.push(END_SYSEX);

        self.transport.write(&buf)
    }

    fn get_i2c_data(&self, _: u8) -> Arc<RwLock<Vec<I2CReply>>> {
        todo!("to be implemented")
    }
}

impl RemoteIo {
    /// Sends a software reset request.
    /// <https://github.com/firmata/protocol/blob/master/protocol.md>
    fn software_reset(&self) -> Result<(), Error> {
        let payload = &[SYSTEM_RESET];
        // trace!("Software reset: {:02X?}", payload);
        self.transport.write(payload)
    }

    /// Starts a conversation with the board: validate the firmware version and...
    fn handshake(&self) -> Result<(), Error> {
        // self.set_connected(false);

        // Forces a software reset: some board do not restart automatically when the connection is opened.
        // Therefore, running two different software in a raw may result to unexpected settings leftover,
        // for instance the report_analog and report_digital on some pins may continue otherwise.
        self.software_reset()?;

        // The RemoteIo protocol is supposed to send the protocol and firmware version automatically,
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
    fn query_firmware(&self) -> Result<(), Error> {
        let payload = &[START_SYSEX, REPORT_FIRMWARE, END_SYSEX];
        // trace!("Query firmware: {:02X?}", payload);
        self.transport.write(payload)
    }

    /// Query the board for all available capabilities.
    fn query_capabilities(&self) -> Result<(), Error> {
        let payload = &[START_SYSEX, CAPABILITY_QUERY, END_SYSEX];
        // trace!("Query capabilities: {:02X?}", payload);
        self.transport.write(payload)
    }

    // ########################################
    // Read/Write on pins

    /// Query the board for available analog pins.
    fn query_analog_mapping(&self) -> Result<(), Error> {
        let payload = &[START_SYSEX, ANALOG_MAPPING_QUERY, END_SYSEX];
        // trace!("Query analog mapping: {:02X?}", payload);
        self.transport.write(payload)
    }

    // ########################################
    // RemoteIo read & handle functions

    /// Read from the protocol, parse and return its type.
    /// The following method should use Firmata protocol such as defined here:
    /// <https://github.com/firmata/protocol/blob/master/protocol.md>
    fn read_and_decode(&self) -> Result<Message, Error> {
        let mut buf = vec![0; 3];
        self.transport.read_exact(&mut buf)?;

        match buf[0] {
            REPORT_PROTOCOL_VERSION => self.handle_protocol_version(&buf),
            ANALOG_MESSAGE..=ANALOG_MESSAGE_BOUND => self.handle_analog_message(&buf),
            DIGITAL_MESSAGE..=DIGITAL_MESSAGE_BOUND => self.handle_digital_message(&buf),
            START_SYSEX => self.handle_sysex_message(&mut buf),
            _ => {
                // trace!("IoPlugin: unexpected data: {:02X?}", buf.as_slice());
                Ok(Message::EmptyResponse)
            }
        }
    }

    /// Handle a REPORT_VERSION_RESPONSE message (0xF9 - return the firmware version).
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#message-types>
    fn handle_protocol_version(&self, buf: &[u8]) -> Result<Message, Error> {
        self.protocol_version.set_with_context(
            format!("{}.{}", buf[1], buf[2]),
            "RemoteIo protocol version",
        )?;

        // trace!("Received protocol version: {}", lock.protocol_version);
        Ok(Message::ReportProtocolVersion)
    }

    /// Handle an ANALOG_MESSAGE message (0xE0 - report state of an analog pin)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion>
    fn handle_analog_message(&self, buf: &[u8]) -> Result<Message, Error> {
        let pin = (buf[0] & 0x0F) + 14;
        let value = (buf[1] as u16) | ((buf[2] as u16) << 7);
        // trace!("Received analog message: pin({})={}", pin, value);
        self.get_pin(pin)?.set_value(value);
        Ok(Message::Analog)
    }

    /// Handle a DIGITAL_MESSAGE message (0x90 - report state of a digital pin/port)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#data-message-expansion>
    fn handle_digital_message(&self, buf: &[u8]) -> Result<Message, Error> {
        let port = buf[0] & 0x0F;
        let value = (buf[1] as u16) | ((buf[2] as u16) << 7);
        // trace!("Received digital message: pin({})={}", port, value);

        for i in 0..8 {
            let pin = (8 * port) + i;
            let pin_instance = self.get_pin(pin)?;
            let mode: PinModeId = pin_instance.get_pin_mode().id;
            if mode == PinModeId::INPUT || mode == PinModeId::PULLUP {
                self.get_pin(pin)?.set_value((value >> (i & 0x07)) & 0x01);
            }
        }
        Ok(Message::Digital)
    }

    /// Handle a START_SYSEX message: dispatch to various message/command/response using the sysex format.
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#sysex-message-format>
    fn handle_sysex_message(&self, buf: &mut Vec<u8>) -> Result<Message, Error> {
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
                // trace!("Sysex: unexpected data: {:02X?}", buf.as_slice());
                Ok(Message::EmptyResponse)
            }
        }
    }

    /// Handle an ANALOG_MAPPING_RESPONSE message (0x6A - reply with analog pins mapping info).
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#analog-mapping-query>
    fn handle_analog_mapping_response(&self, buf: &[u8]) -> Result<Message, Error> {
        let mut i = 2;
        while buf[i] != END_SYSEX {
            let pin = self.get_pin((i - 2) as u8)?;
            match buf[i] {
                SYSEX_REALTIME => {
                    pin.name
                        .set_with_context(format!("D{}", buf[i]), "Pin name ")?;
                    pin.channel.set_with_context(None, "Pin channel")?;
                }
                _ => {
                    pin.set_pin_mode(PinModeId::ANALOG)?;
                    pin.name
                        .set_with_context(format!("A{}", buf[i]), "Pin name ")?;
                    pin.channel.set_with_context(Some(buf[i]), "Pin channel")?;
                }
            }
            i += 1;
        }
        Ok(Message::AnalogMappingResponse)
    }

    /// Handle a CAPABILITY_RESPONSE message (0x6C - reply with supported modes and resolution)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#capability-query>
    fn handle_capability_response(&self, buf: &[u8]) -> Result<Message, Error> {
        let mut id = 0;
        let mut i = 2;
        let mut pins = HashMap::new();

        while buf[i] != END_SYSEX {
            let mut supported_modes: Vec<PinMode> = vec![];

            while buf[i] != SYSEX_REALTIME {
                supported_modes.push(PinMode {
                    id: PinModeId::from(buf[i]),
                    resolution: buf[i + 1],
                });
                i += 2;
            }

            // Add "UNSUPPORTED" as a hack to auto-detach the servos.
            supported_modes.push(PinMode {
                id: PinModeId::UNSUPPORTED,
                resolution: 0,
            });

            let pin = Pin {
                id,
                supported_modes,
                ..Default::default()
            };

            if !pin.supported_modes.is_empty() {
                pin.set_pin_mode(PinModeId::from(pin.supported_modes.first().unwrap().id))?;
            }
            pins.insert(pin.id, Arc::new(pin));

            i += 1;
            id += 1;
        }

        self.pins.set_with_context(pins, "RemoteIo pins")?;

        // trace!("Received capability response: @see hardware.pins");
        Ok(Message::CapabilityResponse)
    }

    /// Handle a REPORT_FIRMWARE message (0x79 - report name and version of the firmware).
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#query-firmware-name-and-version>
    fn handle_firmware_report(&self, buf: &[u8]) -> Result<Message, Error> {
        if buf.len() < 5 {
            return Err(Error::from(ProtocolError::MessageTooShort {
                operation: "handle_firmware_report",
                expected: 5,
                received: buf.len(),
            }));
        }
        let major = buf[2];
        let minor = buf[3];
        self.firmware_version
            .set_with_context(format!("{}.{}", major, minor), "RemoteIo firmware version")?;
        // trace!("Received firmware version: {}", lock.firmware_version);
        if buf.len() > 5 {
            self.firmware_name.set_with_context(
                std::str::from_utf8(&buf[4..buf.len() - 1])?
                    .to_string()
                    .replace('\0', ""),
                "RemoteIo firmware name",
            )?;
            // trace!("Received firmware name: {}", lock.firmware_name);
        }

        Ok(Message::ReportFirmwareVersion)
    }

    /// Handle an I2C_REPLY message (0x6E - read and decode an i2c message)
    /// <https://github.com/firmata/protocol/blob/master/i2c.md>
    fn handle_i2c_reply(&self, buf: &[u8]) -> Result<Message, Error> {
        // trace!("I2C REPLY: {}", format_as_hex(buf));

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
        self.i2c_data.write().push(reply);
        Ok(Message::I2CReply)
    }

    /// Handle a PIN_STATE_RESPONSE message (0x6E - report pin current mode and state)
    /// <https://github.com/firmata/protocol/blob/master/protocol.md#pin-state-query>
    fn handle_pin_state_response(&self, buf: &[u8]) -> Result<Message, Error> {
        let pin = buf[2];
        if buf.len() < 4 || buf[3] == END_SYSEX {
            return Err(Error::from(ProtocolError::MessageTooShort {
                operation: "handle_pin_state_response",
                expected: 5,
                received: buf.len(),
            }));
        }

        // Sets the desired pin mode.
        let pin = self.get_pin(pin)?;
        pin.set_pin_mode(PinModeId::from(buf[3]))?;

        let mut i = 4;
        let mut value: usize = 0;
        while buf[i] != END_SYSEX {
            // Shift value by 7 bits and combine with the next 7 bits
            value = (value << 7) | ((buf[i] as usize) & 0x7F);
            i += 1;
        }
        pin.set_value(value as u16);
        // trace!("Received pin state: {:?}", pin);
        Ok(Message::PinStateResponse)
    }

    /// Manually attaches the board value change listener. This is only used for input events.
    /// This should never be needed unless you manually `detach()` the sensor first for some reason
    /// and want it to start being reactive to events again.
    pub fn start_polling(&self) {
        if self.handler.read().is_none() {
            let self_clone = self.clone();
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

impl Display for RemoteIo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RemoteIo [protocol={} (version={}), firmware={} (version {}), transport={}]",
            self.get_protocol_name(),
            self.get_protocol_version(),
            self.get_firmware_name(),
            self.get_firmware_version(),
            self.transport
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::{create_test_pins, MockTransport};
    use crate::protocols::constants::Message;
    use crate::utils::{format_as_hex, Range};
    use std::sync::Arc;

    fn _get_mock_transport(protocol: &RemoteIo) -> &MockTransport {
        protocol
            .transport
            .as_any()
            .downcast_ref::<MockTransport>()
            .unwrap()
    }

    #[test]
    fn test_creation() {
        let protocol = RemoteIo::default();
        let transport = protocol.transport.as_any().downcast_ref::<Serial>();
        assert!(transport.is_some());

        let protocol = RemoteIo::new("try");
        let transport = protocol.transport.as_any().downcast_ref::<Serial>();
        assert!(transport.is_some());
        assert_eq!(transport.unwrap().get_port(), String::from("try"));

        let protocol = RemoteIo::from(MockTransport::default());
        let transport = protocol.transport.as_any().downcast_ref::<MockTransport>();
        assert!(transport.is_some());
    }

    #[test]
    fn test_software_reset() {
        let protocol = RemoteIo::from(MockTransport::default());

        let result = protocol.software_reset();
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.read().starts_with(&[0xFF]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..1])
        );
    }

    #[test]
    fn test_handshake() {
        let transport = RemoteIo::from(MockTransport::default());
        let result = transport.handshake();
        assert!(result.is_ok(), "{:?}", result);
        let transport = _get_mock_transport(&transport);
        assert!(
            transport.write_buf.read().starts_with(&[
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
        let protocol = RemoteIo::from(MockTransport::default());
        let result = protocol.open();
        assert!(result.is_ok(), "{:?}", result);
    }

    #[test]
    fn test_simple_analog_write() {
        let protocol = RemoteIo::from(MockTransport::default());
        protocol.pins.set(create_test_pins()).unwrap();

        let result = protocol.analog_write(0, 170);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.read().starts_with(&[0xE0, 0x2A, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..3])
        );

        assert_eq!(protocol.get_pin_value(0).unwrap(), 170, "Pin value updated");

        let result = protocol.analog_write(66, 0);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Unknown pin 66."
        );
    }

    #[test]
    fn test_extended_analog_write() {
        let protocol = RemoteIo::from(MockTransport::default());
        protocol.pins.set(create_test_pins()).unwrap();

        // Note1: the pin to use is over 15, so we use extended protocol.
        // Note2: the value sent is over 16384 (0x00004000) so we use multibyte sending.
        let result = protocol.analog_write(22, 17000);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .read()
                .starts_with(&[0xF0, 0x6F, 0x16, 0x68, 0x04, 0x01, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..7])
        );
        assert_eq!(
            protocol.get_pin_value(22).unwrap(),
            17000,
            "Pin value updated"
        );

        let result = protocol.analog_write(42, 0);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Unknown pin 42."
        );
    }

    #[test]
    fn test_digital_write() {
        let protocol = RemoteIo::from(MockTransport::default());
        protocol.pins.set(create_test_pins()).unwrap();

        // TEST
        let result = protocol.digital_write(13, true);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.read().starts_with(&[0x91, 0x7F, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..3])
        );

        assert_eq!(protocol.get_pin_value(13).unwrap(), u16::MAX);
        assert_eq!(
            protocol.get_pin_value(11).unwrap(),
            11,
            "Other pin value does not change"
        );

        let result = protocol.digital_write(66, true);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Unknown pin 66."
        );
    }

    #[test]
    fn test_set_pin_mode() {
        let protocol = RemoteIo::from(MockTransport::default());
        protocol.pins.set(create_test_pins()).unwrap();

        let result = protocol.set_pin_mode(8, PinModeId::OUTPUT);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.read().starts_with(&[0xF4, 0x08, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..3])
        );

        let pin = protocol.get_pin(8).unwrap();
        assert_eq!(pin.get_pin_mode().id, PinModeId::OUTPUT);

        let result = protocol.set_pin_mode(8, PinModeId::SHIFT);
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Pin (8) not compatible with mode (SHIFT)."
        );
    }

    #[test]
    fn test_servo_config() {
        let protocol = RemoteIo::from(MockTransport::default());

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
                .read()
                .starts_with(&[0xF0, 0x70, 0x08, 0x74, 0x03, 0x44, 0x13, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..8])
        );
    }

    #[test]
    fn test_query_firmware() {
        let protocol = RemoteIo::from(MockTransport::default());

        let result = protocol.query_firmware();
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.read().starts_with(&[0xF0, 0x79, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..3])
        );
    }

    #[test]
    fn test_query_capabilities() {
        let protocol = RemoteIo::from(MockTransport::default());

        let result = protocol.query_capabilities();
        assert!(
            result.is_ok(),
            "Query capabilities: {:?}",
            result.unwrap_err()
        );

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.read().starts_with(&[0xF0, 0x6B, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..3])
        );
    }

    #[test]
    fn test_query_analog_mapping() {
        let protocol = RemoteIo::from(MockTransport::default());

        let result = protocol.query_analog_mapping();
        assert!(
            result.is_ok(),
            "Query analog mapping: {:?}",
            result.unwrap_err()
        );

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport.write_buf.read().starts_with(&[0xF0, 0x69, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..3])
        );
    }

    #[test]
    fn test_sampling_interval() {
        let protocol = RemoteIo::from(MockTransport::default());

        let result = protocol.sampling_interval(100);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .read()
                .starts_with(&[0xF0, 0x7A, 0x64, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..5])
        );
    }

    #[hermes_five_macros::test]
    fn test_report_analog() {
        let protocol = RemoteIo::from(MockTransport::default());
        assert_eq!(protocol.analog_reported_channels.load(Ordering::Relaxed), 0);

        // Check data sent when enable reporting
        let result = protocol.report_analog(2, true);
        assert!(result.is_ok(), "{:?}", result);
        let _ = protocol.report_analog(3, true);
        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .read()
                .starts_with(&[0xC2, 0x01, 0xC3, 0x01]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..4])
        );

        // Reporting enables a watch task.
        assert!(protocol.handler.read().is_some());
        assert_eq!(
            protocol.analog_reported_channels.load(Ordering::Relaxed),
            12
        );
        assert!(protocol.is_analog_reporting(2));
        assert!(protocol.is_analog_reporting(3));

        // Remove a report analog keeps the watch task.
        let _ = protocol.report_analog(2, false);
        assert_eq!(protocol.analog_reported_channels.load(Ordering::Relaxed), 8);
        assert!(!protocol.is_analog_reporting(2));
        assert!(protocol.is_analog_reporting(3));
        assert!(protocol.handler.read().is_some());

        // Remove last report analog kills the watch task.
        let _ = protocol.report_analog(3, false);
        assert_eq!(protocol.analog_reported_channels.load(Ordering::Relaxed), 0);
        assert!(!protocol.is_analog_reporting(3));
        assert!(protocol.handler.read().is_none());
    }

    #[hermes_five_macros::test]
    fn test_report_digital() {
        let protocol = RemoteIo::from(MockTransport::default());
        assert_eq!(protocol.digital_reported_pins.load(Ordering::Relaxed), 0);

        // Check data sent when enable reporting
        let result = protocol.report_digital(1, true);
        assert!(result.is_ok(), "{:?}", result);
        let result = protocol.report_digital(13, true);
        assert!(result.is_ok(), "{:?}", result);
        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .read()
                .starts_with(&[0xD0, 0x01, 0xD1, 0x01]), // 0xD0 for port 0 (pin 1-7); 0xD1 for port 1 (pin 8-15)
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..4])
        );

        // Reporting enables a watch task.
        assert!(protocol.handler.read().is_some());
        assert_eq!(
            protocol.digital_reported_pins.load(Ordering::Relaxed),
            0b10000000000010
        );
        assert!(protocol.is_digital_reporting(1));
        assert!(protocol.is_digital_reporting(13));

        // Remove a report analog keeps the watch task.
        let _ = protocol.report_digital(1, false);
        assert_eq!(
            protocol.digital_reported_pins.load(Ordering::Relaxed),
            0b10000000000000
        );
        assert!(!protocol.is_digital_reporting(1));
        assert!(protocol.is_digital_reporting(13));
        assert!(protocol.handler.read().is_some());

        // Remove last report analog kills the watch task.
        let _ = protocol.report_digital(13, false);
        assert_eq!(protocol.digital_reported_pins.load(Ordering::Relaxed), 0);
        assert!(!protocol.is_digital_reporting(1));
        assert!(!protocol.is_digital_reporting(13));
        assert!(protocol.handler.read().is_none());
    }

    #[test]
    fn test_handle_protocol_version() {
        let protocol = RemoteIo::from(MockTransport::new(vec![0xF9, 0x01, 0x19]));

        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report version: {:?}",
            result.unwrap_err()
        );

        assert_eq!(result.unwrap(), Message::ReportProtocolVersion);
        assert_eq!(protocol.protocol_version.get().unwrap(), "1.25");
    }

    #[test]
    fn test_handle_analog_message() {
        let protocol = RemoteIo::from(MockTransport::new(vec![0xE1, 0xDE, 0x00]));
        protocol.pins.set(create_test_pins()).unwrap();

        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle analog message: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::Analog);
        assert_eq!(protocol.get_pin_value(15).unwrap(), 222);
    }

    #[test]
    fn test_handle_digital_message() {
        let protocol = RemoteIo::from(MockTransport::new(vec![0x91, 0x00, 0x00]));
        protocol.pins.set(create_test_pins()).unwrap();
        assert_eq!(protocol.get_pin_value(10).unwrap(), 10);
        assert_eq!(protocol.get_pin_value(12).unwrap(), 12);

        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report version: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::Digital);
        assert_eq!(protocol.get_pin_value(10).unwrap(), 0);
        assert_eq!(protocol.get_pin_value(12).unwrap(), 12);
    }

    #[test]
    fn test_handle_empty_sysex() {
        // Unexpected data when the first byte received in not a valid command.
        let protocol = RemoteIo::from(MockTransport::new(vec![0x11]));
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::EmptyResponse);

        // Unexpected data when the first byte is a sysex, the size is plausible,
        // but the second is not a valid sysex command.
        let protocol = RemoteIo::from(MockTransport::new(vec![0xF0, 0x11, 0x11, 0xF7]));
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::EmptyResponse);

        // Empty command error when a sysex is received and closed immediately.
        let protocol = RemoteIo::from(MockTransport::new(vec![0xF0, 0xF7]));
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
        let protocol = RemoteIo::from(MockTransport::new(vec![0xF0, 0x6A, 0x01, 0x7F, 0x7F, 0xF7]));
        protocol.pins.set(create_test_pins()).unwrap();

        assert_eq!(protocol.get_pin(0).unwrap().get_channel(), None);
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::AnalogMappingResponse);
        assert_eq!(protocol.get_pin(0).unwrap().get_channel(), Some(1));

        // Unsupported possible data
        let protocol = RemoteIo::from(MockTransport::new(vec![0xF0, 0x6A, 0x01, 0x01, 0x01, 0xF7]));
        protocol.pins.set(create_test_pins()).unwrap();
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(
            result.err().unwrap().to_string(),
            "Hardware error: Pin (2) not compatible with mode (ANALOG)."
        );
    }

    #[test]
    fn test_handle_capability_response() {
        let protocol = RemoteIo::from(MockTransport::new(vec![
            0xF0, 0x6C, 0x00, 0x08, 0x7F, 0x00, 0x08, 0x01, 0x08, 0x7F, 0xF7,
        ]));
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle empty sysex: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::CapabilityResponse);
        assert_eq!(protocol.get_pins().len(), 2);
        assert_eq!(protocol.get_pin(0).unwrap().supported_modes.len(), 2);
        assert_eq!(protocol.get_pin(1).unwrap().supported_modes.len(), 3);
    }

    /// Test to decode of "report firmware" command: retrieves the firmware protocol and version.
    #[test]
    fn test_handle_firmware_report() {
        // No firmware name.
        let protocol = RemoteIo::from(MockTransport::new(vec![0xF0, 0x79, 0x01, 0x0C, 0xF7]));
        let result = protocol.read_and_decode();
        assert!(
            result.is_ok(),
            "Handle report firmware: {:?}",
            result.unwrap_err()
        );
        assert_eq!(result.unwrap(), Message::ReportFirmwareVersion);
        assert_eq!(protocol.get_firmware_version(), "1.12");
        assert_eq!(protocol.get_firmware_name(), "N/A");

        // With a custom firmware name.
        let protocol = RemoteIo::from(MockTransport::new(vec![
            0xF0, 0x79, 0x02, 0x40, 0x66, 0x6F, 0x6F, 0x62, 0x61, 0x72, 0xF7,
        ]));
        let result = protocol.read_and_decode();
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap(), Message::ReportFirmwareVersion);
        assert_eq!(protocol.get_firmware_version(), "2.64");
        assert_eq!(protocol.get_firmware_name(), "foobar");

        // Not enough data.
        let protocol = RemoteIo::from(MockTransport::new(vec![0xF0, 0x79, 0x02, 0xF7]));
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(result.err().unwrap().to_string(), "Protocol error: Not enough bytes received - 'handle_firmware_report' expected 5 bytes, 4 received.");
    }

    /// Simulate (and test) the handling of a "pin state response" which is reading at a pin value.
    /// Here, we do check that "reading a new value of 30 on pin 3 now in INPUT mode" will be done
    /// properly.
    #[test]
    fn test_handle_pin_state_response() {
        let protocol = RemoteIo::from(MockTransport::new(vec![
            0xF0, 0x6E, 0x03, 0x00, 0x1E, 0xF7, 0xF0, 0x6E, 0x00, 0xF7,
        ]));
        protocol.pins.set(create_test_pins()).unwrap();
        // By default, the value of pin 3 is 3 and mode is OUTPUT:
        assert_eq!(
            protocol.get_pin(3).unwrap().get_pin_mode().id,
            PinModeId::OUTPUT
        );
        assert_eq!(protocol.get_pin_value(3).unwrap(), 3);

        // Place the command "value of pin 3 changed to 30": read and handle that
        let result = protocol.read_and_decode();
        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap(), Message::PinStateResponse);
        // Now, the value of pin 3 is 30 and mode is INPUT:
        assert_eq!(
            protocol.get_pin(3).unwrap().get_pin_mode().id,
            PinModeId::INPUT
        );
        assert_eq!(protocol.get_pin_value(3).unwrap(), 30);

        // Do the same text wil erroneous incoming data:
        let result = protocol.read_and_decode();
        assert!(result.is_err(), "{:?}", result);
        assert_eq!(result.err().unwrap().to_string(), "Protocol error: Not enough bytes received - 'handle_pin_state_response' expected 5 bytes, 4 received.");
    }

    #[test]
    fn test_i2c_config() {
        let protocol = RemoteIo::from(MockTransport::default());

        let result = protocol.i2c_config(100);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .read()
                .starts_with(&[0xF0, 0x78, 0x64, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..5])
        );
    }

    #[test]
    fn test_i2c_read() {
        let protocol = RemoteIo::from(MockTransport::new(vec![
            0xF0, 0x77, 0x40, 0x00, 0x42, 0x42, 0x42, 0x42, 0xF7, // mock 4 bytes i2c answer.
        ]));

        let result = protocol.i2c_read(0x40, 4); // wait and read 4 bytes.
        assert!(result.is_ok(), "I2C read error: {:?}", result.unwrap_err());

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .read()
                .starts_with(&[0xF0, 0x76, 0x40, 0x08, 0x04, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..7])
        );
    }

    #[test]
    fn test_i2c_write() {
        let protocol = RemoteIo::from(MockTransport::default());

        let result = protocol.i2c_write(0x40, &[0x01, 0x02, 0x03]);
        assert!(result.is_ok(), "{:?}", result);

        let transport = _get_mock_transport(&protocol);
        assert!(
            transport
                .write_buf
                .read()
                .starts_with(&[0xF0, 0x76, 0x40, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0xF7]),
            "Buffer data has been sent [{:?}]",
            format_as_hex(&transport.write_buf.read()[..11])
        );
    }

    // #[test]
    // fn test_handle_i2c_reply() {
    //     // Not enough data.
    //     let protocol = RemoteIo::from(MockTransport::new(vec![0xF0, 0x77, 0x02, 0x02, 0xF7]);
    //
    //     let result = protocol.read_and_decode();
    //     assert!(result.is_err(), "{:?}", result);
    //     assert_eq!(result.err().unwrap().to_string(), "Protocol error: Not enough bytes received - 'handle_i2c_reply' expected 9 bytes, 5 received.");
    //
    //     // Receive an I2C response from i2C address 0x40, register 8, data "coverage".
    //     let protocol = RemoteIo::from(MockTransport::new(vec![
    //         0xF0, 0x77, 0x40, 0x00, 0x08, 0x00, 0x63, 0x00, 0x6F, 0x00, 0x76, 0x00, 0x65, 0x00,
    //         0x72, 0x00, 0x61, 0x00, 0x67, 0x00, 0x65, 0x00, 0xF7,
    //     ]);
    //     let result = protocol.read_and_decode();
    //     assert!(result.is_ok(), "{:?}", result);
    //     {
    //         let data = protocol.get_io().read();
    //         assert_eq!(data.i2c_data.len(), 1);
    //         assert_eq!(data.i2c_data[0].address, 64);
    //         assert_eq!(data.i2c_data[0].register, 8);
    //         let data = data.i2c_data[0].clone().data;
    //         assert_eq!(data, vec![0x63, 0x6F, 0x76, 0x65, 0x72, 0x61, 0x67, 0x65]);
    //         assert_eq!(String::from_utf8_lossy(data.as_slice()), "coverage");
    //     }
    // }

    #[test]
    fn test_debug_and_display() {
        let protocol = RemoteIo::from(MockTransport::default());
        let arc_protocol: Arc<dyn IoProtocol> = Arc::new(protocol);
        // assert_eq!(protocol.get_protocol_name(), "MockProtocol");
        assert_eq!(
            format!("{}", arc_protocol),
            "RemoteIo [protocol=RemoteIo (version=N/A), firmware=N/A (version N/A), transport=MockTransport]"
        )
    }
}
