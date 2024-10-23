use crate::devices::Device;
use crate::errors::Error;
use crate::utils::scale::Scalable;
use crate::utils::{Easing, State};

pub mod digital;
pub mod led;
pub mod servo;

/// A trait for devices that can act on the world: the board "outputs" some state onto them.
///
/// This trait extends [`Device`] and is intended for actuators that requires the same capabilities
/// as devices, including debugging, cloning, and concurrency support.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Output: Device {
    /// Retrieves the actuator current state.
    fn get_state(&self) -> State;
    /// Internal only.
    fn set_state(&mut self, state: State) -> Result<State, Error>;
    /// Retrieves the actuator default (or neutral) state.
    fn get_default(&self) -> State;
    /// Resets the actuator to default (or neutral) state.
    fn reset(&mut self) -> Result<State, Error> {
        self.set_state(self.get_default())
    }
    /// Animates the output of the device. In other word: the state of the device will be animated from
    /// current step to targeted step through an interpolation of in-between states.
    /// The function will last for the required duration and the interpolation will follow an easing
    /// transition function.
    ///
    /// # Arguments
    /// - `state`: the targeted step to meet
    /// - `duration`: the duration (in ms) the animation is expected to last
    /// - `transition`: a transition [`Easing`] function to apply on the state.
    fn animate<S: Into<State>>(&mut self, state: S, duration: u64, transition: Easing)
    where
        Self: Sized;
    /// Indicates the busy status, ie if the device is running an animation.
    fn is_busy(&self) -> bool;
    /// Stops the current animation, if any.
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
}
dyn_clone::clone_trait_object!(Output);

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
