use crate::animations::Easing;
use crate::devices::Device;
use crate::errors::Error;
use crate::utils::State;
use dyn_clone::DynClone;

pub mod digital;
pub mod led;
pub mod pwm;
pub mod servo;

/// A trait for devices that can act on the world: the board "outputs" some state onto them.
///
/// This trait extends [`Device`] and is intended for actuators that requires the same capabilities
/// as devices, including debugging, cloning, and concurrency support.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait OutputDevice: Device + DynClone {
    /// Returns  the actuator current state.
    fn get_state(&self) -> State;

    /// Internal only: you should use the specific device state modifier functions instead.
    fn set_state(&mut self, state: State) -> Result<State, Error>;
    /// Returns  the actuator default (or neutral) state.
    fn get_default(&self) -> State;
    /// Resets the actuator to default (or neutral) state.
    fn reset(&mut self) -> Result<State, Error>;
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
}
dyn_clone::clone_trait_object!(OutputDevice);

#[cfg(test)]
mod tests {
    use crate::mocks::MockOutputDevice;

    use super::*;

    #[test]
    fn test_reset() {
        let mut device = MockOutputDevice::new(42);
        assert_eq!(device.get_state(), State::Integer(42));
        assert!(device.reset().is_ok());
        assert_eq!(device.get_state(), State::Integer(0))
    }
}
