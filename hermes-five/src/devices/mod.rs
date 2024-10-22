use std::fmt::{Debug, Display};

use dyn_clone::DynClone;

// Input re-exports
pub use crate::devices::input::analog::AnalogInput;
pub use crate::devices::input::button::Button;
pub use crate::devices::input::digital::DigitalInput;
pub use crate::devices::input::{Input, InputEvent};
pub use crate::devices::led::Led;
pub use crate::devices::servo::Servo;
pub use crate::devices::servo::ServoType;
use crate::errors::Error;
use crate::utils::scale::Scalable;
use crate::utils::{Easing, State};

mod input;
mod led;
mod servo;

/// A trait for devices that can be debugged, cloned, and used in concurrent contexts.
/// [`Device`] are one of the `Entity` defined in Hermes-Five project: it represents a physical
/// device that is plugged to and can be controlled by a [`Board`]. `Device`s come in two flavor:
/// - `Actuator`: device that can act on the world
/// - `Sensor`: device that can sense or measure data from the world
///
/// Implementors of this trait are required to be `Debug`, `DynClone`, `Send`, and `Sync`.
/// This ensures that devices can be cloned and used safely in multithreaded and async environments.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Device: Debug + Display + DynClone + Send + Sync {}
dyn_clone::clone_trait_object!(Device);

/// A trait for devices that can act on the world, such as adjusting state.
///
/// This trait extends `Device` and adds methods specific to actuators.
///
/// # Methods
///
/// * `set_state(&mut self, state: f64) -> Result<(), Error>`
///     - Sets the actuator's internal state and updates it. Returns an `Error` if the operation fails.
/// * `get_state(&self) -> u16`
///     - Retrieves the current internal state of the device.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Output: Device {
    fn animate<S: Into<State>>(&mut self, state: S, duration: u64, transition: Easing)
    where
        Self: Sized;
    fn stop(&mut self);
    /// Internal only.
    fn scale_state(&mut self, previous: State, target: State, progress: f32) -> State {
        match target {
            State::Integer(value) => {
                State::Integer(progress.scale(0, 1, previous.as_integer(), value))
            }
            State::Signed(value) => {
                State::Signed(progress.scale(0, 1, previous.as_signed_integer(), value))
            }
            State::Float(value) => State::Float(progress.scale(0, 1, previous.as_float(), value)),
            _ => match progress {
                0.0 => previous,
                _ => target,
            },
        }
    }
    /// Internal only.
    fn set_state(&mut self, state: State) -> Result<State, Error>;
    /// Retrieves the actuator current state.
    fn get_state(&self) -> State;
    /// Retrieves the actuator default (or neutral) state.
    fn get_default(&self) -> State;
    /// Indicates the busy status, ie if the device is running an animation.
    fn is_busy(&self) -> bool;
    /// Resets the actuator to default (or neutral) state.
    fn reset(&mut self) -> Result<State, Error> {
        self.set_state(self.get_default())
    }
}
dyn_clone::clone_trait_object!(Output);

#[cfg(feature = "serde")]
pub mod arc_rwlock_serde {
    use std::sync::Arc;

    use parking_lot::RwLock;
    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S, T>(val: &Arc<RwLock<T>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        T::serialize(&*val.read(), s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Arc<RwLock<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Ok(Arc::new(RwLock::new(T::deserialize(d)?)))
    }

    #[cfg(test)]
    mod arc_rwlock_serde_tests {
        use serde_json;

        use crate::mocks::output::MockOutputDevice;

        #[test]
        fn test_serialize() {
            let test = MockOutputDevice::new(20);

            let serialized = serde_json::to_string(&test);
            assert!(serialized.is_ok());

            let expected_json = r#"{"state":20,"lock":42}"#;
            assert_eq!(serialized.unwrap(), expected_json);
        }

        #[test]
        fn test_deserialize() {
            let json_data = r#"{"state":20,"lock":42}"#;
            let deserialized = serde_json::from_str::<MockOutputDevice>(json_data);

            assert!(deserialized.is_ok());
            assert_eq!(deserialized.unwrap().get_locked_value(), 42);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mocks::output::MockOutputDevice;

    use super::*;

    #[test]
    fn test_scale_state_integer() {
        let mut device = MockOutputDevice::new(0);

        // Halfway between 10 and 20
        let result = device.scale_state(State::Integer(10), State::Integer(20), 0.5);
        assert_eq!(result, State::Integer(15));

        // 75% between 10 and 20
        let result = device.scale_state(State::Integer(10), State::Integer(20), 0.75);
        assert_eq!(result, State::Integer(18));

        // 120% between 10 and 20
        let result = device.scale_state(State::Integer(10), State::Integer(20), 1.2);
        assert_eq!(result, State::Integer(22));
    }

    #[test]
    fn test_scale_state_signed() {
        let mut device = MockOutputDevice::new(0);

        // Halfway between 10 and 20
        let result = device.scale_state(State::Signed(-10), State::Signed(10), 0.5);
        assert_eq!(result, State::Signed(0));

        // 75% between 10 and 20
        let result = device.scale_state(State::Signed(-10), State::Signed(10), 0.75);
        assert_eq!(result, State::Signed(5));

        // 120% between 10 and 20
        let result = device.scale_state(State::Signed(-10), State::Signed(10), 1.2);
        assert_eq!(result, State::Signed(14));
    }

    #[test]
    fn test_scale_state_float() {
        let mut device = MockOutputDevice::new(0);

        // Halfway between 10 and 20
        let result = device.scale_state(State::Float(1.0), State::Float(2.0), 0.5);
        assert_eq!(result, State::Float(1.5));

        // 75% between 10 and 20
        let result = device.scale_state(State::Float(1.0), State::Float(2.0), 0.75);
        assert_eq!(result, State::Float(1.75));

        // 120% between 10 and 20
        let result = device.scale_state(State::Float(1.0), State::Float(2.0), 1.2);
        assert_eq!(result, State::Float(2.200000047683716));
    }

    #[test]
    fn test_scale_state_non_numeric() {
        let mut device = MockOutputDevice::new(0);

        let result = device.scale_state(State::Boolean(false), State::Boolean(true), 0.0);
        assert_eq!(result, State::Boolean(false));

        let result = device.scale_state(State::Boolean(false), State::Boolean(true), 0.5);
        assert_eq!(result, State::Boolean(true));
    }

    #[test]
    fn test_reset() {
        let mut device = MockOutputDevice::new(42);
        assert_eq!(device.get_state(), State::Integer(42));
        assert!(device.reset().is_ok());
        assert_eq!(device.get_state(), State::Integer(0))
    }
}
