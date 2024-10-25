use std::fmt::{Display, Formatter};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::animations::{Animation, Keyframe, Track};
use crate::devices::{Device, Output};
use crate::errors::HardwareError::IncompatibleMode;
use crate::errors::{Error, StateError};
use crate::hardware::Board;
use crate::io::{Pin, PinIdOrName, PinModeId, PluginIO};
use crate::utils::{Easing, State};

/// Represents an analog actuator of unspecified type: an [`Output`] [`Device`] that write analog values from a PWM compatible pin.
/// https://docs.arduino.cc/language-reference/en/functions/analog-io/analogWrite/
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct PwmOutput {
    // ########################################
    // # Basics
    /// The pin (id) of the [`Board`] used to control the output value.
    pin: u16,
    /// The current output state.
    #[cfg_attr(feature = "serde", serde(with = "crate::devices::arc_rwlock_serde"))]
    state: Arc<RwLock<u16>>,
    /// The output default value (default: 0).
    default: u16,

    // ########################################
    // # Volatile utility data.
    /// Caches the max output value depending on resolution.
    #[cfg_attr(feature = "serde", serde(skip))]
    max_value: u16,
    /// The protocol used by the board to communicate with the device.
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol: Box<dyn PluginIO>,
    /// Inner handler to the task running the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    animation: Arc<Option<Animation>>,
}

impl PwmOutput {
    /// Creates an instance of a [`PwmOutput`] attached to a given board.
    ///
    /// # Parameters
    /// * `board`: the [`Board`] which the PwmOutput is attached to.
    /// * `pin`: the output pin used to write PWM output value.
    /// * `default`: the default output value taken by this device.
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the pin does not support PWM mode.
    pub fn new<T: Into<PinIdOrName>>(board: &Board, pin: T, default: u16) -> Result<Self, Error> {
        let pin = board.get_io().get_pin(pin)?.clone();

        let mut output = Self {
            pin: pin.id,
            state: Arc::new(RwLock::new(default)),
            default,
            max_value: 0,
            protocol: board.get_protocol(),
            animation: Arc::new(None),
        };

        // Set pin mode to PWM.
        output.protocol.set_pin_mode(output.pin, PinModeId::PWM)?;

        // Retrieve PWM max value for the pin.
        output.max_value = board
            .get_data()
            .read()
            .get_pin(pin.id)?
            .get_max_possible_value();

        // Resets the output to default value.
        output.reset()?;

        Ok(output)
    }

    /// Sets the PWM value.
    ///
    /// # Parameters
    /// * `value`: the value to set
    pub fn set_value(&mut self, value: u16) -> Result<&Self, Error> {
        self.set_state(value.into())?;
        Ok(self)
    }

    /// Sets the PWM value to a percentage of its max value.
    /// NOTE: everything above 100 is considered 100%.
    ///
    /// # Parameters
    /// * `percentage`: the percentage of the value to set
    pub fn set_percentage(&mut self, percentage: u8) -> Result<&Self, Error> {
        let percentage = percentage.min(100) as u16;
        let value = (percentage * self.max_value) / 100;
        self.set_state(value.into())?;
        Ok(self)
    }

    // ########################################
    // Setters and Getters.

    /// Retrieves the PIN (id) used to control the LED.
    pub fn get_pin(&self) -> u16 {
        self.pin
    }

    /// Retrieves [`Pin`] information.
    pub fn get_pin_info(&self) -> Result<Pin, Error> {
        let lock = self.protocol.get_data().read();
        Ok(lock.get_pin(self.pin)?.clone())
    }

    /// Gets the current PWM value.
    pub fn get_value(&self) -> u16 {
        *self.state.read()
    }

    /// Gets the current percentage of the PWM value compared to max possible.
    pub fn get_percentage(&self) -> u8 {
        let value = *self.state.read();
        ((value as f32 * 100.0) / self.max_value as f32).round() as u8
    }
}

impl Display for PwmOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PwmOutput (pin={}) [state={} ({}%), default={}]",
            self.pin,
            self.state.read(),
            self.get_percentage(),
            self.default,
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for PwmOutput {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Output for PwmOutput {
    /// Retrieves the actuator current state.
    fn get_state(&self) -> State {
        (*self.state.read()).into()
    }

    /// Internal only: Update the LED to the target state.
    ///
    /// /!\ You should rather use [`Led::on()`], [`Led::off()`], [`Led::set_brightness()`]` functions.`
    fn set_state(&mut self, state: State) -> Result<State, Error> {
        let value = match state {
            State::Integer(value) => Ok(value as u16),
            State::Signed(value) => match value >= 0 {
                true => Ok(value as u16),
                false => Err(StateError),
            },
            State::Float(value) => match value >= 0.0 {
                true => Ok(value as u16),
                false => Err(StateError),
            },
            _ => Err(StateError),
        }?;

        match self.get_pin_info()?.mode.id {
            PinModeId::PWM => self.protocol.analog_write(self.pin, value),
            id => Err(Error::from(IncompatibleMode {
                mode: id,
                pin: self.pin,
                context: "update pwm output",
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
    /// This does not necessarily turn off the servo: it will remain in its current state when stopped.
    fn stop(&mut self) {
        if let Some(animation) = Arc::get_mut(&mut self.animation).and_then(Option::as_mut) {
            animation.stop();
        }
        self.animation = Arc::new(None);
    }
}

#[cfg(test)]
mod tests {
    use crate::devices::output::pwm::PwmOutput;
    use crate::devices::Output;
    use crate::hardware::Board;
    use crate::io::PinModeId;
    use crate::mocks::plugin_io::MockPluginIO;
    use crate::pause;
    use crate::utils::{Easing, State};

    #[test]
    fn test_creation() {
        let board = Board::from(MockPluginIO::default());

        // Default LOW state.
        let output = PwmOutput::new(&board, 8, 0).unwrap();
        assert_eq!(output.get_pin(), 8);
        assert_eq!(*output.state.read(), 0);
        assert_eq!(output.get_state().as_integer(), 0);
        assert_eq!(output.get_default().as_integer(), 0);

        // Default HIGH state.
        let output = PwmOutput::new(&board, 8, 50).unwrap();
        assert_eq!(output.get_pin(), 8);
        assert_eq!(*output.state.read(), 50);
        assert_eq!(output.get_state().as_integer(), 50);
        assert_eq!(output.get_default().as_integer(), 50);

        // Created from pin name
        let output = PwmOutput::new(&board, "D11", 50).unwrap();
        assert_eq!(output.get_pin(), 11);
    }

    #[test]
    fn test_set_value() {
        let mut output = PwmOutput::new(&Board::from(MockPluginIO::default()), 8, 0).unwrap();
        output.set_value(127).unwrap();
        assert_eq!(*output.state.read(), 127);
        assert_eq!(output.get_value(), 127);
    }

    #[test]
    fn test_set_percent() {
        let mut output = PwmOutput::new(&Board::from(MockPluginIO::default()), 8, 0).unwrap();
        output.set_percentage(50).unwrap();
        assert_eq!(*output.state.read(), 127);
        assert_eq!(output.get_value(), 127);
        assert_eq!(output.get_percentage(), 50);
        output.set_percentage(200).unwrap();
        assert_eq!(*output.state.read(), 0xFF);
        assert_eq!(output.get_value(), 255);
        assert_eq!(output.get_percentage(), 100);
    }

    #[test]
    fn test_set_state() {
        let mut output = PwmOutput::new(&Board::from(MockPluginIO::default()), 11, 127).unwrap();
        assert!(output.set_state(State::Integer(0)).is_ok());
        assert_eq!(*output.state.read(), 0);
        assert!(output.set_state(State::Integer(127)).is_ok());
        assert_eq!(*output.state.read(), 127);

        assert!(output.set_state(State::Signed(0)).is_ok());
        assert_eq!(*output.state.read(), 0);
        assert!(output.set_state(State::Signed(127)).is_ok());
        assert_eq!(*output.state.read(), 127);
        assert!(output.set_state(State::Signed(-42)).is_err());

        assert!(output.set_state(State::Float(0.0)).is_ok());
        assert_eq!(*output.state.read(), 0);
        assert!(output.set_state(State::Float(127.0)).is_ok());
        assert_eq!(*output.state.read(), 127);
        assert!(output.set_state(State::Float(-42.0)).is_err());

        assert!(output
            .set_state(State::String(String::from("incorrect format")))
            .is_err()); // Should return an error due to incompatible state
                        // Force an incompatible pin mode
        let _ = output
            .protocol
            .set_pin_mode(output.pin, PinModeId::UNSUPPORTED);
        assert!(output.set_state(State::Integer(1)).is_err()); // Should return an error due to incompatible pin mode.
    }

    #[test]
    fn test_get_pin_info() {
        let output = PwmOutput::new(&Board::from(MockPluginIO::default()), 11, 20).unwrap();
        let pin_info = output.get_pin_info();
        assert!(pin_info.is_ok());
        assert_eq!(pin_info.unwrap().id, 11);
    }

    #[hermes_macros::test]
    fn test_animation() {
        let mut output = PwmOutput::new(&Board::from(MockPluginIO::default()), 11, 20).unwrap();
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
        let mut output = PwmOutput::new(&Board::from(MockPluginIO::default()), 11, 212).unwrap();
        let _ = output.set_value(127);
        let display_str = format!("{}", output);
        assert_eq!(
            display_str,
            "PwmOutput (pin=11) [state=127 (50%), default=212]"
        );
    }
}
