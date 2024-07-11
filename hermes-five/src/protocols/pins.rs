use std::fmt::Debug;

use crate::protocols::constants::*;

/// The current state and configuration of a pin.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pin {
    /// The pin id: should correspond also to the position of the pin in the `Vec<Pin>`
    pub id: u8,
    /// Currently configured mode.
    pub mode: u8,
    /// Current resolution.
    pub resolution: u8,
    /// All pin supported modes.
    pub supported_modes: Vec<u8>,
    /// Pin value.
    pub value: i32,
}
impl Default for Pin {
    fn default() -> Self {
        Self {
            id: 0,
            mode: PIN_MODE_ANALOG,
            supported_modes: vec![PIN_MODE_ANALOG],
            resolution: DEFAULT_ANALOG_RESOLUTION,
            value: 0,
        }
    }
}

impl Debug for Pin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Transformer for "resolution"
        let resolution_str = format!("{} bits", self.resolution);

        // Transformer for "mode"
        let mode_owned;
        let mode_str = match self.mode {
            PIN_MODE_INPUT => "input mode",
            PIN_MODE_OUTPUT => "output mode",
            PIN_MODE_ANALOG => "analog pin",
            PIN_MODE_PWM => "pwn",
            PIN_MODE_SERVO => "servo",
            PIN_MODE_SHIFT => "shift register",
            PIN_MODE_I2C => "I2C",
            PIN_MODE_ONEWIRE => "1-Wire",
            PIN_MODE_STEPPER => "stepper",
            PIN_MODE_ENCODER => "encoder",
            PIN_MODE_SERIAL => "serial",
            PIN_MODE_SPI => "SPI sensor",
            PIN_MODE_SONAR => "sonar/proximity sensor",
            PIN_MODE_TONE => "piezzo buzzer",
            PIN_MODE_DHT => "DHT sensor",
            PIN_MODE_IGNORE => "unsupported",
            x => {
                mode_owned = format!("unknown: {}", x);
                &mode_owned
            }
        };

        f.debug_struct("Pin")
            .field("id", &self.id)
            .field("mode", &mode_str)
            .field("supported modes", &self.supported_modes)
            .field("analog resolution", &resolution_str)
            .field("value", &self.value)
            .finish()
    }
}
