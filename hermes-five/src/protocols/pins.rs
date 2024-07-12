use std::fmt::Debug;

use crate::protocols::Error;

/// The current state and configuration of a pin.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pin {
    /// The pin id: should correspond also to the position of the pin in the `Vec<Pin>`
    pub id: u8,
    /// Currently configured mode.
    pub mode: PinMode,
    /// Current resolution.
    pub resolution: u8,
    /// All pin supported modes.
    pub supported_modes: Vec<PinMode>,
    /// For analog pin, this is the channel number ie "A0", "A1", etc...
    // @todo convert this to ID accepting both u8 and &str (3 or "A3")
    pub channel: Option<u8>,
    /// Pin value.
    pub value: i32,
}

impl Debug for Pin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Transformer for "resolution"
        let resolution_str = format!("{} bits", self.resolution);
        let mode_str = format!("{:?}", self.mode);
        let supported_modes_str = format!("{:?}", self.supported_modes);
        let channel_str = match self.channel {
            None => String::default(),
            Some(ch) => format!("A{}", ch),
        };

        f.debug_struct("Pin")
            .field("id", &self.id)
            .field("mode", &mode_str)
            .field("supported modes", &supported_modes_str)
            .field("resolution", &resolution_str)
            .field("channel", &channel_str)
            .field("value", &self.value)
            .finish()
    }
}

// ########################################

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PinMode {
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
    IGNORE = 0x7F,
}

impl PinMode {
    pub fn from_u8(value: u8) -> Result<PinMode, Error> {
        match value {
            0 => Ok(PinMode::INPUT),
            1 => Ok(PinMode::OUTPUT),
            2 => Ok(PinMode::ANALOG),
            3 => Ok(PinMode::PWM),
            4 => Ok(PinMode::SERVO),
            5 => Ok(PinMode::SHIFT),
            6 => Ok(PinMode::I2C),
            7 => Ok(PinMode::ONEWIRE),
            8 => Ok(PinMode::STEPPER),
            9 => Ok(PinMode::ENCODER),
            0x0A => Ok(PinMode::SERIAL),
            0x0B => Ok(PinMode::PULLUP),
            0x0C => Ok(PinMode::SPI),
            0x0D => Ok(PinMode::SONAR),
            0x0E => Ok(PinMode::TONE),
            0x0F => Ok(PinMode::DHT),
            0x7F => Ok(PinMode::IGNORE),
            x => Err(Error::Custom {
                info: format!("Pin mode does not exist: {}", x),
            }),
        }
    }
}

impl From<PinMode> for u8 {
    fn from(mode: PinMode) -> u8 {
        mode as u8
    }
}

// ########################################

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PinValue {
    /// LOW value for a digital pin.
    LOW = 0,
    /// HIGH value for a digital pin.
    HIGH = 1,
    /// PWM max resolution value.
    #[allow(non_camel_case_types)]
    MAX_PWM = 255,
}

impl PinValue {
    pub fn from_u8(value: u8) -> Result<PinValue, Error> {
        match value {
            0 => Ok(PinValue::LOW),
            1 => Ok(PinValue::HIGH),
            x => Err(Error::Custom {
                info: format!("Pin value does not exist: {}", x),
            }),
        }
    }
}

impl From<PinValue> for u8 {
    fn from(value: PinValue) -> u8 {
        value as u8
    }
}
