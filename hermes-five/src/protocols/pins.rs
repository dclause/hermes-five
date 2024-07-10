use crate::protocols::firmata::*;

/// The current state and configuration of a pin.
#[derive(Debug, Clone)]
pub struct Pin {
    /// Currently configured mode.
    pub mode: u8,
    /// Current resolution.
    pub resolution: u8,
    /// All pin modes.
    pub modes: Vec<u8>,
    /// Pin value.
    pub value: i32,
}
impl Default for Pin {
    fn default() -> Self {
        Self {
            mode: PIN_MODE_ANALOG,
            modes: vec![PIN_MODE_ANALOG],
            resolution: DEFAULT_ANALOG_RESOLUTION,
            value: 0,
        }
    }
}
