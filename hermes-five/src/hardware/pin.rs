use crate::errors::HardwareError::IncompatiblePin;
use crate::errors::*;
use std::fmt::{Debug, Display, Formatter};
use std::sync::atomic::{AtomicU16, AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

/// Defines an I2C reply.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct I2CReply {
    pub address: u8,
    pub register: u8,
    pub data: Vec<u8>,
}

/// Represents the current state and configuration of a pin.
#[derive(Clone, Default)]
pub struct Pin {
    /// The pin ID, which also corresponds to the index of the [`IoData::pins`] hashmap.
    pub id: u8,
    /// The pin name: an alternative String representation of the pin name: 'D13', 'A0', 'GPIO13' for instance.
    pub name: Arc<OnceLock<String>>,
    /// Currently configured mode.
    pub mode: Arc<AtomicU8>,
    /// All pin supported modes.
    pub supported_modes: Vec<PinMode>,
    /// For analog pin, this is the channel number ie "A0"=>0, "A1"=>1, etc...
    pub channel: Arc<OnceLock<Option<u8>>>,
    /// Pin value.
    pub value: Arc<AtomicU16>,
}

impl Pin {
    /// Gets the currently configured [PinMode] of this pin.
    ///
    /// [`PinMode`]: crate::PinMode  
    pub fn get_pin_mode(&self) -> PinMode {
        let current_mode_id = PinModeId::from(&self.mode);
        self.supported_modes
            .iter()
            .find(|_mode| _mode.id == current_mode_id)
            .cloned()
            .unwrap()
    }

    /// Sets the mode of this pin to the given [`PinModeId`], if supported.
    ///
    /// # Arguments
    /// * `mode` - The desired [`PinModeId`] to assign to this pin (e.g. `Output`, `Input`, `Pwm`, etc.).
    ///
    /// # Returns
    /// The assigned [`PinMode`] on success.
    ///
    /// # Errors
    /// Returns [`Error::IncompatiblePin`] if the requested mode is not listed in `supported_modes`.
    ///
    /// [`PinModeId`]: crate::PinModeId  
    /// [`PinMode`]: crate::PinMode  
    /// [`Error::IncompatiblePin`]: crate::Error::IncompatiblePin
    pub fn set_pin_mode(&self, mode: PinModeId) -> Result<PinMode, Error> {
        let pin = self.id;
        let _mode = self
            .supported_modes
            .iter()
            .find(|_mode| _mode.id == mode)
            .cloned()
            .ok_or(HardwareError::IncompatiblePin { pin, mode })?;
        self.mode.store(_mode.id as u8, Ordering::Relaxed);
        Ok(_mode)
    }

    /// Ensures the pin is currently in the specified [`PinModeId`].
    ///
    /// This function validates the pin’s active mode against the expected one, and returns an error
    /// if the modes do not match. This is useful for enforcing correct operation modes before I/O.
    ///
    /// # Arguments
    /// * `mode` - The mode ID that the pin is expected to be in.
    ///
    /// # Errors
    /// Returns [`Error::IncompatiblePin`] if the current mode does not match the expected one.
    ///
    /// [`PinModeId`]: crate::PinModeId
    /// [`Error::IncompatiblePin`]: crate::Error::IncompatiblePin
    pub fn ensure_mode_is(&self, mode: PinModeId) -> Result<(), Error> {
        let id = PinModeId::from(&self.mode);
        match id == mode {
            true => Ok(()),
            false => Err(IncompatiblePin {
                mode: id,
                pin: self.id,
            }),
        }?;
        Ok(())
    }

    /// Returns the [`PinMode`] supported by this pin for the given [`PinModeId`], if any.
    ///
    /// This checks whether the pin can operate in the specified mode and returns the
    /// corresponding [`PinMode`] if it's supported.
    ///
    /// # Arguments
    ///
    /// * `mode` - The identifier of the mode to check for support.
    ///
    /// # Returns
    ///
    /// * `Some(PinMode)` if the mode is supported by the pin.
    /// * `None` if the mode is not supported.
    ///
    /// [`PinModeId`]: crate::PinModeId
    /// [`PinMode`]: crate::PinMode
    pub fn is_supported(&self, mode: PinModeId) -> Option<PinMode> {
        self.supported_modes
            .iter()
            .find(|_mode| _mode.id == mode)
            .cloned()
    }

    /// Retrieves the pin value.
    #[inline(always)]
    pub fn get_value(&self) -> u16 {
        self.value.load(Ordering::Relaxed)
    }

    /// Sets the pin value.
    #[inline(always)]
    pub fn set_value(&self, value: u16) {
        self.value.store(value, Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn get_name(&self) -> &str {
        self.name.get().map(|s| s.as_str()).unwrap_or("")
    }

    #[inline(always)]
    pub fn get_channel(&self) -> Option<u8> {
        match self.channel.get() {
            Some(Some(ch)) => Some(*ch),
            _ => None,
        }
    }

    /// Get the max value this pin can reach.
    ///
    /// This is defined by the resolution of the current pin mode.
    pub fn get_max_possible_value(&self) -> u16 {
        self.get_pin_mode().get_max_possible_value()
    }
}

impl Debug for Pin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Transformer for "resolution"
        let mode_str = format!("{}", self.get_pin_mode());

        let mut debug_struct = f.debug_struct("Pin");
        debug_struct
            .field("id", &self.id)
            .field("name", &self.get_name())
            .field("mode", &mode_str)
            .field("supported modes", &self.supported_modes);
        if let Some(channel) = self.get_channel() {
            debug_struct.field("channel", &channel);
        } else {
            debug_struct.field("channel", &None::<u8>);
        }
        debug_struct.field("value", &self.value).finish()
    }
}

// ########################################

/// Represents a pin identifier, which can be either a numeric ID (e.g., `1`)
/// or a name (e.g., `"D1"`, `"A1"`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub enum PinIdOrName {
    Id(u8),
    Name(String),
}

impl From<u8> for PinIdOrName {
    fn from(n: u8) -> Self {
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Copy)]
pub struct PinMode {
    /// Currently configured mode.
    pub id: PinModeId,
    /// Resolution (number of bits) this mode uses.
    pub resolution: u8,
}

impl PinMode {
    /// Returns the maximum value this pin mode can represent, based on its resolution.
    ///
    /// This is computed as `(2^resolution) - 1`. For example:
    /// - 8-bit resolution → 255
    /// - 10-bit resolution → 1023
    ///
    /// # Example
    /// ```
    /// use hermes_five::hardware::{PinMode, PinModeId};
    /// assert_eq!(PinMode { id: PinModeId::OUTPUT, resolution: 8 }.get_max_possible_value(), 255);
    /// ```
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
    #[inline(always)]
    fn from_u8(value: u8) -> Self {
        match value {
            0 => PinModeId::INPUT,
            1 => PinModeId::OUTPUT,
            2 => PinModeId::ANALOG,
            3 => PinModeId::PWM,
            4 => PinModeId::SERVO,
            5 => PinModeId::SHIFT,
            6 => PinModeId::I2C,
            7 => PinModeId::ONEWIRE,
            8 => PinModeId::STEPPER,
            9 => PinModeId::ENCODER,
            0x0A => PinModeId::SERIAL,
            0x0B => PinModeId::PULLUP,
            0x0C => PinModeId::SPI,
            0x0D => PinModeId::SONAR,
            0x0E => PinModeId::TONE,
            0x0F => PinModeId::DHT,
            0x7F => PinModeId::UNSUPPORTED,
            _ => PinModeId::UNSUPPORTED,
        }
    }
}

impl From<u8> for PinModeId {
    #[inline(always)]
    fn from(n: u8) -> Self {
        PinModeId::from_u8(n)
    }
}

impl From<PinModeId> for Arc<AtomicU8> {
    #[inline(always)]
    fn from(mode: PinModeId) -> Self {
        Arc::new(AtomicU8::new(mode as u8))
    }
}

impl From<&Arc<AtomicU8>> for PinModeId {
    #[inline(always)]
    fn from(value: &Arc<AtomicU8>) -> Self {
        PinModeId::from_u8(value.load(Ordering::Relaxed))
    }
}

impl Display for PinModeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use crate::hardware::{PinIdOrName, PinMode, PinModeId};
    use crate::mocks::{create_analog_pin, create_pwm_pin};
    use crate::utils::ArcOnceLockExt;
    use hermes_five::mocks::create_shift_pin;
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_pin_ensure_mode_is() {
        let pin = create_pwm_pin(2, 90);

        // Mode is supported
        let supported_mode = pin.ensure_mode_is(PinModeId::PWM);
        assert!(supported_mode.is_ok());

        // Mode is not supported
        assert!(pin.ensure_mode_is(PinModeId::DHT).is_err());
    }

    #[test]
    fn test_pin_mode_max_value() {
        let pin_mode = PinMode {
            id: PinModeId::INPUT,
            resolution: 8,
        };
        assert_eq!(pin_mode.get_max_possible_value(), 255);
        let pin_mode = PinMode {
            id: PinModeId::PWM,
            resolution: 10,
        };
        assert_eq!(pin_mode.get_max_possible_value(), 1023);
    }

    #[test]
    fn test_pin_debug() {
        let pin = create_shift_pin(2, 100);
        pin.name.set_with_context(String::from("D2"), "").unwrap();
        assert_eq!(format!("{:?}", pin), String::from("Pin { id: 2, name: \"D2\", mode: \"SHIFT\", supported modes: [[UNSUPPORTED], [id: SHIFT, resolution: 8], [id: OUTPUT, resolution: 1]], channel: None, value: 100 }"));
        let pin = create_analog_pin(10, 50);
        pin.name.set_with_context(String::from("A10"), "").unwrap();
        assert_eq!(format!("{:?}", pin), String::from("Pin { id: 10, name: \"A10\", mode: \"ANALOG\", supported modes: [[UNSUPPORTED], [id: ANALOG, resolution: 8], [id: INPUT, resolution: 1], [id: OUTPUT, resolution: 1]], channel: 10, value: 50 }"));
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
        assert_eq!(PinModeId::from_u8(0), PinModeId::INPUT);
        assert_eq!(PinModeId::from_u8(1), PinModeId::OUTPUT);
        assert_eq!(PinModeId::from_u8(2), PinModeId::ANALOG);
        assert_eq!(PinModeId::from_u8(3), PinModeId::PWM);
        assert_eq!(PinModeId::from_u8(4), PinModeId::SERVO);
        assert_eq!(PinModeId::from_u8(5), PinModeId::SHIFT);
        assert_eq!(PinModeId::from_u8(6), PinModeId::I2C);
        assert_eq!(PinModeId::from_u8(7), PinModeId::ONEWIRE);
        assert_eq!(PinModeId::from_u8(8), PinModeId::STEPPER);
        assert_eq!(PinModeId::from_u8(9), PinModeId::ENCODER);
        assert_eq!(PinModeId::from_u8(0x0A), PinModeId::SERIAL);
        assert_eq!(PinModeId::from_u8(0x0B), PinModeId::PULLUP);
        assert_eq!(PinModeId::from_u8(0x0C), PinModeId::SPI);
        assert_eq!(PinModeId::from_u8(0x0D), PinModeId::SONAR);
        assert_eq!(PinModeId::from_u8(0x0E), PinModeId::TONE);
        assert_eq!(PinModeId::from_u8(0x0F), PinModeId::DHT);
        assert_eq!(PinModeId::from_u8(0x7F), PinModeId::UNSUPPORTED);
        assert_eq!(PinModeId::from_u8(0xFF), PinModeId::UNSUPPORTED);

        // From Arc<AtomicU8>> to PinModeId
        assert_eq!(
            PinModeId::from(&Arc::new(AtomicU8::new(13))),
            PinModeId::SONAR
        );
        // From PinModeId to Arc<AtomicU8>>
        let converted: Arc<AtomicU8> = PinModeId::DHT.into();
        assert_eq!(converted.load(Ordering::Relaxed), 15);

        // From PinModeId to u8
        assert_eq!(PinModeId::SHIFT as u8, 5);
        assert_eq!(PinModeId::SPI as u8, 12);
    }

    #[test]
    fn test_pin_mode_id_display() {
        assert_eq!(format!("{}", PinModeId::PWM), "PWM");
    }

    #[test]
    fn test_pin_id_from() {
        let pin = PinIdOrName::from(42);
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
