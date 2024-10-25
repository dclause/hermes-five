use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use crate::errors::HardwareError::{IncompatibleMode, UnknownPin};
use crate::errors::*;

/// Represents the internal data that a [`IoProtocol`] handles.
///
/// This struct is hidden behind an `Arc<RwLock<IoData>>` to allow safe concurrent access
/// and modification through the `IoData` type. It encapsulates data relevant
/// to the protocol, such as pins and I2C communication data.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IoData {
    /// All `Pin` instances, representing the hardware's pins.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub pins: HashMap<u16, Pin>,
    /// A vector of `I2CReply` instances, representing I2C communication data.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub i2c_data: Vec<I2CReply>,
    /// List pins with digital reporting activated.
    pub digital_reported_pins: Vec<u16>,
    /// List pins with analog reporting activated.
    pub analog_reported_channels: Vec<u8>,
    /// A string indicating the version of the protocol.
    pub protocol_version: String,
    /// A string representing the name of the firmware.
    pub firmware_name: String,
    /// A string representing the version of the firmware.
    pub firmware_version: String,
    /// A boolean indicating whether the IoProtocol is connected.
    pub connected: bool,
}

impl IoData {
    /// Retrieves a reference to a pin by its id or name.
    ///
    /// # Arguments
    /// * `pin`  - The index of the pin to retrieve.
    ///
    /// # Errors
    /// * `UnknownPin` - An `Error` returned if the pin index is out of bounds.
    pub fn get_pin<T: Into<PinIdOrName>>(&self, pin: T) -> Result<&Pin, Error> {
        let pin = pin.into();
        match &pin {
            PinIdOrName::Id(id) => self.pins.get(id).ok_or(Error::from(UnknownPin { pin })),
            PinIdOrName::Name(name) => Ok(self
                .pins
                .iter()
                .find(|(_, pin)| pin.name == *name)
                .ok_or(Error::from(UnknownPin { pin }))?
                .1),
        }
    }

    /// Retrieves a mutable reference to a pin by its id or name.
    ///
    /// # Arguments
    /// * `pin` - The index of the pin to retrieve.
    ///
    /// # Errors
    /// * `UnknownPin` - An `Error` returned if the pin index is out of bounds.
    pub fn get_pin_mut<T: Into<PinIdOrName>>(&mut self, pin: T) -> Result<&mut Pin, Error> {
        let pin = pin.into();
        match &pin {
            PinIdOrName::Id(id) => self.pins.get_mut(id).ok_or(Error::from(UnknownPin { pin })),
            PinIdOrName::Name(name) => Ok(self
                .pins
                .iter_mut()
                .find(|(_, &mut ref pin)| pin.name == *name)
                .ok_or(Error::from(UnknownPin { pin }))?
                .1),
        }
    }
}

/// Defines an I2C reply.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct I2CReply {
    pub address: u16,
    pub register: u16,
    pub data: Vec<u16>,
}

/// Represents the current state and configuration of a pin.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default)]
pub struct Pin {
    /// The pin ID, which also corresponds to the index of the [`IoData::pins`] hashmap.
    pub id: u16,
    /// The pin name: an alternative String representation of the pin name: 'D13', 'A0', 'GPIO13' for instance.
    pub name: String,
    /// Currently configured mode.
    pub mode: PinMode,
    /// All pin supported modes.
    pub supported_modes: Vec<PinMode>,
    /// For analog pin, this is the channel number ie "A0"=>0, "A1"=>1, etc...
    pub channel: Option<u8>,
    /// Pin value.
    pub value: u16,
}

impl Pin {
    /// Verifies if a pin supports the given mode and returns it if it does.
    ///
    /// # Arguments
    /// * `mode`: The ID of the mode to retrieve.
    ///
    /// # Returns
    /// * `None` if the mode is not supported.
    /// * `PinMode` the `PinMode` configuration if supported
    pub fn supports_mode(&self, mode: PinModeId) -> Option<PinMode> {
        self.supported_modes.iter().find(|m| m.id == mode).copied()
    }

    /// Validates that the pin is in the given mode.
    ///
    /// # Arguments
    /// * `mode`: The ID of the mode to check: the pin should be in that mode.
    ///
    /// # Errors
    /// *`IncompatibleMode`: the pin's current mode does not match the expected mode.
    pub fn validate_current_mode(&self, mode: PinModeId) -> Result<(), Error> {
        match self.mode.id == mode {
            true => Ok(()),
            false => Err(IncompatibleMode {
                mode: self.mode.id,
                pin: self.id,
                context: "check_current_mode",
            }),
        }?;
        Ok(())
    }

    /// Get the max value this pin can reach.
    ///
    /// This is defined by the resolution of the current pin mode.
    pub fn get_max_possible_value(&self) -> u16 {
        self.mode.get_max_possible_value()
    }
}

impl Debug for Pin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Transformer for "resolution"
        let mode_str = format!("{}", self.mode);

        let mut debug_struct = f.debug_struct("Pin");
        debug_struct
            .field("id", &self.id)
            .field("name", &self.name)
            .field("mode", &mode_str)
            .field("supported modes", &self.supported_modes);
        if let Some(channel) = self.channel {
            debug_struct.field("channel", &channel);
        } else {
            debug_struct.field("channel", &None::<u8>);
        }
        debug_struct.field("value", &self.value).finish()
    }
}

// ########################################

/// Defines a structure to receive either an id or a name for a pin: 1, 'D1' or 'A1' for instance.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub enum PinIdOrName {
    Id(u16),
    Name(String),
}

impl From<u16> for PinIdOrName {
    fn from(n: u16) -> Self {
        PinIdOrName::Id(n)
    }
}

impl From<&str> for PinIdOrName {
    fn from(s: &str) -> Self {
        PinIdOrName::Name(s.to_string())
    }
}

impl From<String> for PinIdOrName {
    fn from(s: String) -> Self {
        PinIdOrName::Name(s)
    }
}

impl Display for PinIdOrName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PinIdOrName::Id(n) => write!(f, "{}", n),
            PinIdOrName::Name(s) => write!(f, "{:?}", s),
        }
    }
}

// ########################################

/// Represents a mode configuration for a pin.
///
/// # Fields
/// - `id`: The ID of the mode.
/// - `resolution`: The resolution (number of bits) this mode uses.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Copy)]
pub struct PinMode {
    /// Currently configured mode.
    pub id: PinModeId,
    /// Resolution (number of bits) this mode uses.
    pub resolution: u8,
}

impl PinMode {
    /// Get the max value this pinMode can reach according to its resolution.
    pub fn get_max_possible_value(&self) -> u16 {
        (1 << self.resolution) - 1
    }
}

impl Display for PinMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Debug for PinMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.id {
            PinModeId::UNSUPPORTED => write!(f, "[{}]", self.id),
            _ => write!(f, "[id: {}, resolution: {}]", self.id, self.resolution),
        }
    }
}

// ########################################

/// Enumerates the possible modes for a pin.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
#[repr(u8)]
pub enum PinModeId {
    /// Same as INPUT defined in Arduino.
    INPUT = 0,
    /// Same as OUTPUT defined in Arduino.h
    OUTPUT = 1,
    /// Analog pin in analogInput mode
    ANALOG = 2,
    /// Digital pin in PWM output mode
    PWM = 3,
    /// Digital pin in Servo output mode
    SERVO = 4,
    /// shiftIn/shiftOut mode
    SHIFT = 5,
    /// Pin included in I2C setup
    I2C = 6,
    /// Pin configured for 1-wire
    ONEWIRE = 7,
    /// Pin configured for stepper motor
    STEPPER = 8,
    /// Pin configured for rotary encoders
    ENCODER = 9,
    /// Pin configured for serial communication
    SERIAL = 0x0A,
    /// Enable internal pull-up resistor for pin
    PULLUP = 0x0B,
    /// Pin configured for SPI
    SPI = 0x0C,
    /// Pin configured for proximity sensors
    SONAR = 0x0D,
    /// Pin configured for piezzo buzzer tone generation
    TONE = 0x0E,
    /// Pin configured for DHT humidity and temperature sensors
    DHT = 0x0F,
    /// Pin configured to be ignored by digitalWrite and capabilityResponse
    #[default]
    UNSUPPORTED = 0x7F,
}

impl PinModeId {
    /// Converts a `u8` byte value into a `PinModeId`.
    ///
    /// # Arguments
    /// * `value`: The `u8` value representing the pin mode.
    ///
    /// # Errors
    /// * `Unknown`: The value does not match any known pin mode.
    ///
    /// # Returns
    /// The corresponding `PinModeId` if the value is valid, otherwise returns an error.
    pub fn from_u8(value: u8) -> Result<PinModeId, Error> {
        match value {
            0 => Ok(PinModeId::INPUT),
            1 => Ok(PinModeId::OUTPUT),
            2 => Ok(PinModeId::ANALOG),
            3 => Ok(PinModeId::PWM),
            4 => Ok(PinModeId::SERVO),
            5 => Ok(PinModeId::SHIFT),
            6 => Ok(PinModeId::I2C),
            7 => Ok(PinModeId::ONEWIRE),
            8 => Ok(PinModeId::STEPPER),
            9 => Ok(PinModeId::ENCODER),
            0x0A => Ok(PinModeId::SERIAL),
            0x0B => Ok(PinModeId::PULLUP),
            0x0C => Ok(PinModeId::SPI),
            0x0D => Ok(PinModeId::SONAR),
            0x0E => Ok(PinModeId::TONE),
            0x0F => Ok(PinModeId::DHT),
            0x7F => Ok(PinModeId::UNSUPPORTED),
            x => Err(UnknownError {
                info: format!("PinMode not found with value: {}", x),
            }),
        }
    }
}

impl From<PinModeId> for u8 {
    fn from(mode: PinModeId) -> u8 {
        mode as u8
    }
}

impl Display for PinModeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Pin, PinIdOrName, PinMode, PinModeId};
    use crate::mocks::create_test_plugin_io_data;

    #[test]
    fn test_get_pin_success() {
        assert_eq!(create_test_plugin_io_data().get_pin(3).unwrap().value, 3);
        assert_eq!(create_test_plugin_io_data().get_pin(11).unwrap().value, 11);
        assert_eq!(
            create_test_plugin_io_data().get_pin_mut(3).unwrap().value,
            3
        );
        assert_eq!(
            create_test_plugin_io_data().get_pin_mut(11).unwrap().value,
            11
        );
    }

    #[test]
    fn test_get_pin_error() {
        assert!(create_test_plugin_io_data().get_pin(66).is_err());
        assert!(create_test_plugin_io_data().get_pin_mut(66).is_err());
    }

    #[test]
    fn test_mutate_pin() {
        let mut hardware = create_test_plugin_io_data();
        assert_eq!(hardware.get_pin_mut(11).unwrap().value, 11);
        hardware.get_pin_mut(11).unwrap().value = 255;
        assert_eq!(hardware.get_pin_mut(11).unwrap().value, 255);
    }

    #[test]
    fn test_pin_supports_mode() {
        let pin = Pin {
            supported_modes: vec![
                PinMode {
                    id: PinModeId::INPUT,
                    resolution: 0,
                },
                PinMode {
                    id: PinModeId::OUTPUT,
                    resolution: 0,
                },
            ],
            ..Default::default()
        };

        // Mode is supported
        let supported_mode = pin.supports_mode(PinModeId::INPUT);
        assert!(supported_mode.is_some());

        // Mode is not supported
        assert!(pin.supports_mode(PinModeId::PWM).is_none());
    }

    #[test]
    fn test_pin_mode_max_value() {
        let pin_mode = PinMode {
            id: PinModeId::INPUT,
            resolution: 8,
        };

        assert_eq!(pin_mode.get_max_possible_value(), 255);
    }

    #[test]
    fn test_check_current_mode_success() {
        let pin = Pin {
            mode: PinMode {
                id: PinModeId::PWM,
                resolution: 10,
            },
            ..Default::default()
        };

        assert!(pin.validate_current_mode(PinModeId::PWM).is_ok());
        assert!(pin.validate_current_mode(PinModeId::SHIFT).is_err());
        assert_eq!(pin.get_max_possible_value(), 1023);
    }

    #[test]
    fn test_pin_display() {
        let mut pin = Pin {
            supported_modes: vec![
                PinMode {
                    id: PinModeId::INPUT,
                    resolution: 0,
                },
                PinMode {
                    id: PinModeId::OUTPUT,
                    resolution: 1,
                },
                PinMode {
                    id: PinModeId::ANALOG,
                    resolution: 8,
                },
            ],
            channel: Some(1),
            ..Default::default()
        };
        assert_eq!(format!("{:?}", pin), String::from("Pin { id: 0, name: \"\", mode: \"UNSUPPORTED\", supported modes: [[id: INPUT, resolution: 0], [id: OUTPUT, resolution: 1], [id: ANALOG, resolution: 8]], channel: 1, value: 0 }"));
        pin.mode = PinMode {
            id: PinModeId::INPUT,
            resolution: 0,
        };
        pin.channel = None;
        assert_eq!(format!("{:?}", pin), String::from("Pin { id: 0, name: \"\", mode: \"INPUT\", supported modes: [[id: INPUT, resolution: 0], [id: OUTPUT, resolution: 1], [id: ANALOG, resolution: 8]], channel: None, value: 0 }"));
    }

    #[test]
    fn test_pin_mode_display() {
        let mode = PinMode {
            id: PinModeId::PWM,
            resolution: 8,
        };
        assert_eq!(format!("{}", mode), "PWM");
    }

    #[test]
    fn test_pin_mode_debug() {
        let mode = PinMode {
            id: PinModeId::PWM,
            resolution: 8,
        };
        assert_eq!(format!("{:?}", mode), "[id: PWM, resolution: 8]");
        let unsupported = PinMode {
            id: PinModeId::UNSUPPORTED,
            resolution: 0,
        };
        assert_eq!(format!("{:?}", unsupported), "[UNSUPPORTED]");
    }

    #[test]
    fn test_pin_mode_id_conversions() {
        // From u8 to PinModeId: success
        let mode = PinModeId::from_u8(0x0F);
        assert!(mode.is_ok());
        assert_eq!(PinModeId::from_u8(0).unwrap(), PinModeId::INPUT);
        assert_eq!(PinModeId::from_u8(1).unwrap(), PinModeId::OUTPUT);
        assert_eq!(PinModeId::from_u8(2).unwrap(), PinModeId::ANALOG);
        assert_eq!(PinModeId::from_u8(3).unwrap(), PinModeId::PWM);
        assert_eq!(PinModeId::from_u8(4).unwrap(), PinModeId::SERVO);
        assert_eq!(PinModeId::from_u8(5).unwrap(), PinModeId::SHIFT);
        assert_eq!(PinModeId::from_u8(6).unwrap(), PinModeId::I2C);
        assert_eq!(PinModeId::from_u8(7).unwrap(), PinModeId::ONEWIRE);
        assert_eq!(PinModeId::from_u8(8).unwrap(), PinModeId::STEPPER);
        assert_eq!(PinModeId::from_u8(9).unwrap(), PinModeId::ENCODER);
        assert_eq!(PinModeId::from_u8(0x0A).unwrap(), PinModeId::SERIAL);
        assert_eq!(PinModeId::from_u8(0x0B).unwrap(), PinModeId::PULLUP);
        assert_eq!(PinModeId::from_u8(0x0C).unwrap(), PinModeId::SPI);
        assert_eq!(PinModeId::from_u8(0x0D).unwrap(), PinModeId::SONAR);
        assert_eq!(PinModeId::from_u8(0x0E).unwrap(), PinModeId::TONE);
        assert_eq!(PinModeId::from_u8(0x0F).unwrap(), PinModeId::DHT);
        assert_eq!(PinModeId::from_u8(0x7F).unwrap(), PinModeId::UNSUPPORTED);

        // From u8 to PinModeId: error
        let error_mode = PinModeId::from_u8(100);
        assert!(error_mode.is_err());
        assert_eq!(
            error_mode.err().unwrap().to_string(),
            "Unknown error: PinMode not found with value: 100."
        );

        // From PinModeId to u8
        assert_eq!(u8::from(PinModeId::SHIFT), 5);
    }

    #[test]
    fn test_pin_mode_id_display() {
        assert_eq!(format!("{}", PinModeId::PWM), "PWM");
    }

    #[test]
    fn test_pin_id_from() {
        let pin = PinIdOrName::from(42u16);
        assert_eq!(pin, PinIdOrName::Id(42));
        let pin: PinIdOrName = 4.into();
        assert_eq!(pin, PinIdOrName::Id(4));
        let pin = PinIdOrName::from("D1");
        assert_eq!(pin, PinIdOrName::Name("D1".to_string()));
        let pin = PinIdOrName::from("A1".to_string());
        assert_eq!(pin, PinIdOrName::Name("A1".to_string()));
    }

    #[test]
    fn test_pin_id_display() {
        let pin = PinIdOrName::Id(42);
        assert_eq!(pin.to_string(), "42");
        let pin = PinIdOrName::Name(String::from("A0"));
        assert_eq!(pin.to_string(), "\"A0\"");
    }
}
