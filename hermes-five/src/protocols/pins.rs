use std::fmt::{Debug, Display, Formatter};

use crate::protocols::{Error, IncompatibleMode};

/// The current state and configuration of a pin.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default)]
pub struct Pin {
    /// The pin id: should correspond also to the position of the pin in the [`ProtocolHardware::pins`]
    pub id: u16,
    /// Currently configured mode.
    pub mode: PinMode,
    /// All pin supported modes.
    pub supported_modes: Vec<PinMode>,
    /// For analog pin, this is the channel number ie "A0", "A1", etc...
    pub channel: Option<u8>,
    /// Pin value.
    pub value: u16,
}

impl Pin {
    /// Retrieve the given mode among the available ones.
    pub fn get_mode(&self, mode: PinModeId) -> Option<PinMode> {
        match self.supported_modes.iter().find(|m| m.id == mode) {
            None => None,
            Some(mode) => Some(mode.clone()),
        }
    }

    /// Check if pin currently have
    pub fn check_current_mode(&self, mode: PinModeId) -> Result<(), Error> {
        match self.mode.id == mode {
            true => Ok(()),
            false => Err(IncompatibleMode {
                mode: self.mode.id,
                pin: self.id,
                operation: String::from("digital_write requires OUTPUT mode"),
            }),
        }
    }
}

impl Debug for Pin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Transformer for "resolution"
        let mode_str = format!("{}", self.mode);
        let channel_str = match self.channel {
            None => String::default(),
            Some(ch) => format!("A{}", ch),
        };

        f.debug_struct("Pin")
            .field("id", &self.id)
            .field("mode", &mode_str)
            .field("supported modes", &self.supported_modes)
            .field("channel", &channel_str)
            .field("value", &self.value)
            .finish()
    }
}

// ########################################

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default)]
pub struct PinMode {
    /// Currently configured mode.
    pub id: PinModeId,
    /// Resolution (number of bits) this mode uses.
    pub resolution: u8,
}

impl Display for PinMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Debug for PinMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.id {
            PinModeId::UNSUPPORTED => write!(f, "{}", self.id),
            _ => write!(f, "[id: {}, resolution: {}]", self.id, self.resolution),
        }
    }
}

// ########################################

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
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
            x => Err(Error::Custom {
                info: format!("Pin mode does not exist: {}", x),
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
