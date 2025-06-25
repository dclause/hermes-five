use crate::devices::OutputDevice;
use crate::errors::{Error, HardwareError, StateError};
use crate::hardware::{Hardware, LowLevelApiExt, Pin, PinIdOrName, PinModeId};
use crate::protocols::IoProtocol;
use crate::utils::State;
use hermes_five_macros::output_device;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Represents a digital actuator of unspecified type: an [`OutputDevice`] that write digital values
/// from an OUTPUT compatible pin.
/// <https://docs.arduino.cc/language-reference/en/functions/digital-io/digitalwrite/>
#[output_device(bool)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct DigitalOutput {
    // ########################################
    // # Basics
    /// The pin (id) of the [`Board`] used to control the output value.
    #[cfg_attr(feature = "serde", serde(rename = "pin"))]
    id: u8,
    /// The current output state.
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::serde_arc_atomic"))]
    state: Arc<AtomicBool>,

    // ########################################
    // # Volatile utility data.
    /// The pin on the [`Board`] used to control the device.
    #[cfg_attr(feature = "serde", serde(skip))]
    pin: Arc<Pin>,
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::utils::serde_arc_protocol", skip_serializing)
    )]
    protocol: Arc<dyn IoProtocol>,
}

impl DigitalOutput {
    /// Creates an instance of a [`DigitalOutput`] attached to a given board.
    ///
    /// # Errors
    /// * `HardwareError::UnknownPin`: this function will bail an error if the pin does not exist for this board.
    /// * `HardwareError::IncompatiblePin`: this function will bail an error if the pin does not support OUTPUT mode.
    pub fn new<T: Into<PinIdOrName>>(
        board: &dyn Hardware,
        pin: T,
        default: bool,
    ) -> Result<Self, Error> {
        let pin = board.get_pin(pin)?;

        let mut output = Self {
            id: pin.id,
            pin: pin.clone(),
            state: Arc::new(AtomicBool::new(default)),
            default,
            protocol: board.get_protocol(),
            animation: Arc::new(None),
        };

        // Set pin mode to OUTPUT.
        output.protocol.set_pin_mode(pin.id, PinModeId::OUTPUT)?;

        // Resets the output to default value.
        output.reset()?;

        Ok(output)
    }

    /// Turn the output HIGH.
    pub fn turn_on(&mut self) -> Result<&Self, Error> {
        self.set_state(State::Boolean(true))?;
        Ok(self)
    }

    /// Turn the output LOW.
    pub fn turn_off(&mut self) -> Result<&Self, Error> {
        self.set_state(State::Boolean(false))?;
        Ok(self)
    }

    /// Toggle the current state, if on then turn off, if off then turn on.
    pub fn toggle(&mut self) -> Result<&Self, Error> {
        match self.is_high() {
            true => self.turn_off(),
            false => self.turn_on(),
        }
    }

    // ########################################
    // Setters and Getters.

    /// Returns the pin (id) used by the device.
    pub fn get_id(&self) -> u8 {
        self.pin.id
    }

    /// Returns the pin (id) used by the device.
    pub fn get_pin(&self) -> Arc<Pin> {
        self.pin.clone()
    }

    /// Indicates if the device state is HIGH.
    pub fn is_high(&self) -> bool {
        self.get_value()
    }

    /// Indicates if the device state is LOW.
    pub fn is_low(&self) -> bool {
        !self.get_value()
    }

    #[inline(always)]
    fn parse_state(&self, state: State) -> Result<bool, Error> {
        match state {
            State::Boolean(value) => Ok(value),
            State::Integer(value) => match value {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(StateError),
            },
            _ => Err(StateError),
        }
    }

    #[inline(always)]
    fn apply_value(&mut self, value: bool) -> Result<(), Error> {
        match PinModeId::from(&self.pin.mode) {
            // on/off digital operation.
            PinModeId::OUTPUT => self.protocol.digital_write(self.pin.id, value),
            id => Err(Error::from(HardwareError::IncompatiblePin {
                mode: id,
                pin: self.pin.id,
            })),
        }
    }
    #[inline(always)]
    fn get_value(&self) -> bool {
        self.state.load(Ordering::Relaxed)
    }
    #[inline(always)]
    fn set_value(&self, value: bool) {
        self.state.store(value, Ordering::Relaxed)
    }
}

impl Display for DigitalOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DigitalOutput (pin={}) [state={}, default={}]",
            self.pin.id,
            self.get_value(),
            self.default,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::animations::Easing;
    use crate::devices::output::digital::DigitalOutput;
    use crate::devices::OutputDevice;
    use crate::hardware::{Board, LowLevelApiExt, PinModeId};
    use crate::mocks::MockProtocol;
    use crate::pause;
    use crate::utils::{ArcOnceLockExt, State};

    #[test]
    fn test_creation() {
        let board = Board::new(MockProtocol::default());

        // Default LOW state.
        let output = DigitalOutput::new(&board, 13, false).unwrap();
        assert_eq!(output.get_id(), 13);
        assert!(!output.get_value());
        assert!(!output.get_state().as_bool());
        assert!(!output.get_default().as_bool());
        assert!(output.is_low());
        assert!(!output.is_high());

        // Default HIGH state.
        let output = DigitalOutput::new(&board, 4, true).unwrap();
        assert_eq!(output.get_id(), 4);
        assert!(output.get_value());
        assert!(output.get_state().as_bool());
        assert!(output.get_default().as_bool());
        assert!(output.is_high());
        assert!(!output.is_low());

        // Created from pin name
        board
            .get_pin(13)
            .unwrap()
            .name
            .set_with_context(String::from("custom_name"), "")
            .unwrap();
        let output = DigitalOutput::new(&board, "custom_name", true).unwrap();
        assert_eq!(output.get_id(), 13);
    }

    #[test]
    fn test_set_high() {
        let mut output =
            DigitalOutput::new(&Board::new(MockProtocol::default()), 4, false).unwrap();
        output.turn_on().unwrap();
        assert!(output.turn_on().is_ok());
        assert!(output.get_value());
    }

    #[test]
    fn test_set_low() {
        let mut output = DigitalOutput::new(&Board::new(MockProtocol::default()), 5, true).unwrap();
        assert!(output.turn_off().is_ok());
        assert!(!output.get_value());
    }

    #[test]
    fn test_toggle() {
        let mut output =
            DigitalOutput::new(&Board::new(MockProtocol::default()), 5, false).unwrap();
        assert!(output.toggle().is_ok()); // Toggle to HIGH
        assert!(output.get_value());
        assert!(output.toggle().is_ok()); // Toggle to LOW
        assert!(!output.get_value());
    }

    #[test]
    fn test_set_state() {
        let mut output =
            DigitalOutput::new(&Board::new(MockProtocol::default()), 13, false).unwrap();
        assert!(output.set_state(State::Boolean(true)).is_ok());
        assert!(output.get_value());
        assert!(output.set_state(State::Boolean(false)).is_ok());
        assert!(!output.get_value());

        assert!(output.set_state(State::Integer(1)).is_ok());
        assert!(output.get_value());
        assert!(output.set_state(State::Integer(0)).is_ok());
        assert!(!output.get_value());
        assert!(output.set_state(State::Integer(42)).is_err());

        assert!(output
            .set_state(State::String(String::from("incorrect format")))
            .is_err()); // Should return an error due to incompatible state
                        // Force an incompatible pin mode
        let _ = output
            .protocol
            .set_pin_mode(output.pin.id, PinModeId::UNSUPPORTED)
            .is_ok();
        assert!(output.set_state(State::Boolean(true)).is_err()); // Should return an error due to incompatible pin mode.
    }

    #[hermes_five_macros::test]
    fn test_animation() {
        let mut output =
            DigitalOutput::new(&Board::new(MockProtocol::default()), 13, false).unwrap();
        assert!(!output.is_busy());
        // Stop something not started should not fail.
        output.stop();
        // This animation does not make sense !
        output.animate(true, 500, Easing::Linear);
        pause!(100);
        assert!(output.is_busy()); // Animation is currently running.
        output.stop();
    }

    #[test]
    fn test_display_impl() {
        let mut output =
            DigitalOutput::new(&Board::new(MockProtocol::default()), 13, true).unwrap();
        let _ = output.turn_off();
        let display_str = format!("{}", output);
        assert_eq!(
            display_str,
            "DigitalOutput (pin=13) [state=false, default=true]"
        );
    }
}
