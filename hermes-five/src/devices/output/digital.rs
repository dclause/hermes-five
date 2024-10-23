use std::fmt::{Display, Formatter};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::animation::{Animation, Keyframe, Track};
use crate::board::Board;
use crate::devices::{Device, Output};
use crate::errors::HardwareError::IncompatibleMode;
use crate::errors::{Error, StateError};
use crate::protocols::{Pin, PinIdOrName, PinModeId, Protocol};
use crate::utils::{Easing, State};

/// Represents a digital actuator of unspecified type: an [`Output`] [`Device`] that write digital values
/// from an OUTPUT compatible pin.
/// https://docs.arduino.cc/built-in-examples/digital/DigitalInput
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct DigitalOutput {
    // ########################################
    // # Basics
    /// The pin (id) of the [`Board`] used to control the output value.
    pin: u16,
    /// The current output state.
    #[cfg_attr(feature = "serde", serde(with = "crate::devices::arc_rwlock_serde"))]
    state: Arc<RwLock<bool>>,
    /// The output default value (default: 0).
    default: bool,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol: Box<dyn Protocol>,
    /// Inner handler to the task running the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    animation: Arc<Option<Animation>>,
}

impl DigitalOutput {
    /// Creates an instance of a [`DigitalOutput`] attached to a given board.
    ///
    /// # Parameters
    /// * `board`: the [`Board`] which the DigitalOutput is attached to.
    /// * `pin`: the output pin used to write digital output value.
    /// * `default`: the default output value taken by this device.
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the pin does not support OUTPUT mode.
    pub fn new<T: Into<PinIdOrName>>(board: &Board, pin: T, default: bool) -> Result<Self, Error> {
        let pin = board.get_hardware().get_pin(pin)?.clone();

        let mut output = Self {
            pin: pin.id,
            state: Arc::new(RwLock::new(default)),
            default,
            protocol: board.get_protocol(),
            animation: Arc::new(None),
        };

        // Set pin mode to OUTPUT.
        output
            .protocol
            .set_pin_mode(output.pin, PinModeId::OUTPUT)?;

        // Resets the output to default value.
        //output.reset()?;

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

    /// Retrieves the PIN (id) used to control the LED.
    pub fn get_pin(&self) -> u16 {
        self.pin
    }

    /// Retrieves [`Pin`] information.
    pub fn get_pin_info(&self) -> Result<Pin, Error> {
        let lock = self.protocol.get_hardware().read();
        Ok(lock.get_pin(self.pin)?.clone())
    }

    /// Indicates if the device state is HIGH.
    pub fn is_high(&self) -> bool {
        self.state.read().clone()
    }

    /// Indicates if the device state is LOW.
    pub fn is_low(&self) -> bool {
        !self.state.read().clone()
    }
}

impl Display for DigitalOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DigitalOutput (pin={}) [state={}, default={}]",
            self.pin,
            self.state.read(),
            self.default,
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for DigitalOutput {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Output for DigitalOutput {
    /// Retrieves the actuator current state.
    fn get_state(&self) -> State {
        self.state.read().clone().into()
    }

    /// Internal only: Update the LED to the target state.
    ///
    /// /!\ You should rather use [`Led::on()`], [`Led::off()`], [`Led::set_brightness()`]` functions.`
    fn set_state(&mut self, state: State) -> Result<State, Error> {
        let value = match state {
            State::Boolean(value) => Ok(value),
            State::Integer(value) => match value {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(Error::from(StateError)),
            },
            _ => Err(Error::from(StateError)),
        }?;

        match self.get_pin_info()?.mode.id {
            // on/off digital operation.
            PinModeId::OUTPUT => self.protocol.digital_write(self.pin, value),
            id => Err(Error::from(IncompatibleMode {
                mode: id,
                pin: self.pin,
                context: "update digital output",
            })),
        }?;
        *self.state.write() = value;
        Ok(value.into())
    }

    /// Retrieves the actuator default (or neutral) state.
    fn get_default(&self) -> State {
        self.default.into()
    }

    /// Animates the actuator state.
    fn animate<S: Into<State>>(&mut self, state: S, duration: u64, transition: Easing) {
        let mut animation = Animation::from(
            Track::new(self.clone())
                .with_keyframe(Keyframe::new(state, 0, duration).set_transition(transition)),
        );
        animation.play();
        self.animation = Arc::new(Some(animation));
    }

    /// Indicates the busy status, ie if the device is running an animation.
    fn is_busy(&self) -> bool {
        self.animation.is_some()
    }

    /// Stops the current animation.
    /// This does not necessarily turn off the LED;
    /// it will remain in its current state when stopped.
    fn stop(&mut self) {
        if let Some(animation) = Arc::get_mut(&mut self.animation).and_then(Option::as_mut) {
            animation.stop();
        }
        self.animation = Arc::new(None);
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::devices::output::digital::DigitalOutput;
    use crate::devices::Output;
    use crate::mocks::protocol::MockProtocol;
    use crate::pause;
    use crate::protocols::PinModeId;
    use crate::utils::{Easing, State};

    fn _setup_led(pin: u16) -> DigitalOutput {
        let board = Board::from(MockProtocol::default()); // Assuming a mock Board implementation
        DigitalOutput::new(&board, pin, false).unwrap()
    }

    #[test]
    fn test_creation() {
        let board = Board::from(MockProtocol::default());

        // Default LOW state.
        let output = DigitalOutput::new(&board, 13, false).unwrap();
        assert_eq!(output.get_pin(), 13);
        assert_eq!(*output.state.read(), false);
        assert!(output.is_low());
        assert!(!output.is_high());

        // Default HIGH state.
        let output = DigitalOutput::new(&board, 4, true).unwrap();
        assert_eq!(output.get_pin(), 4);
        assert_eq!(*output.state.read(), true);
        assert!(output.is_high());
        assert!(!output.is_low());

        // Created from pin name
        let output = DigitalOutput::new(&board, "D13", true).unwrap();
        assert_eq!(output.get_pin(), 13);

        // Created for a ANALOG pin.
        let output = DigitalOutput::new(&board, "A14", false).unwrap();
        assert_eq!(output.get_pin(), 14);
    }

    #[test]
    fn test_set_high() {
        let mut output =
            DigitalOutput::new(&Board::from(MockProtocol::default()), 4, false).unwrap();
        output.turn_on().unwrap();
        assert!(output.turn_on().is_ok());
        assert_eq!(*output.state.read(), true);
    }

    #[test]
    fn test_set_low() {
        let board = Board::from(MockProtocol::default());
        let mut output = DigitalOutput::new(&board, 5, true).unwrap();
        assert!(output.turn_off().is_ok());
        assert_eq!(*output.state.read(), false);
    }

    #[test]
    fn test_toggle() {
        let mut output =
            DigitalOutput::new(&Board::from(MockProtocol::default()), 5, false).unwrap();
        assert!(output.toggle().is_ok()); // Toggle to HIGH
        assert_eq!(*output.state.read(), true);
        assert!(output.toggle().is_ok()); // Toggle to LOW
        assert_eq!(*output.state.read(), false);
    }

    #[test]
    fn test_set_state() {
        let mut output =
            DigitalOutput::new(&Board::from(MockProtocol::default()), 13, false).unwrap();
        assert!(output.set_state(State::Boolean(true)).is_ok());
        assert_eq!(*output.state.read(), true);
        assert!(output.set_state(State::Boolean(false)).is_ok());
        assert_eq!(*output.state.read(), false);

        assert!(output.set_state(State::Integer(1)).is_ok());
        assert_eq!(*output.state.read(), true);
        assert!(output.set_state(State::Integer(0)).is_ok());
        assert_eq!(*output.state.read(), false);

        assert!(output
            .set_state(State::String(String::from("incorrect format")))
            .is_err()); // Should return an error due to incompatible state
                        // Force an incompatible pin mode
        let _ = output
            .protocol
            .set_pin_mode(output.pin, PinModeId::UNSUPPORTED);
        assert!(output.set_state(State::Boolean(false)).is_err()); // Should return an error due to incompatible pin mode.
    }

    #[test]
    fn test_get_pin_info() {
        let output = DigitalOutput::new(&Board::from(MockProtocol::default()), 13, false).unwrap();
        let pin_info = output.get_pin_info();
        assert!(pin_info.is_ok());
        assert_eq!(pin_info.unwrap().id, 13);
    }

    #[hermes_macros::test]
    fn test_animation() {
        let mut output =
            DigitalOutput::new(&Board::from(MockProtocol::default()), 13, false).unwrap();
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
            DigitalOutput::new(&Board::from(MockProtocol::default()), 13, true).unwrap();
        let _ = output.turn_off();
        let display_str = format!("{}", output);
        assert_eq!(
            display_str,
            "DigitalOutput (pin=13) [state=false, default=true]"
        );
    }
}
