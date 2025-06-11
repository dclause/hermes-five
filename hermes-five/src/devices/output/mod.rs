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
    fn reset(&mut self) -> Result<State, Error> {
        self.stop();
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

}
dyn_clone::clone_trait_object!(OutputDevice);

mod sealed {
    use crate::animations::Animation;
    use crate::devices::Device;
    use crate::errors::Error;
    use crate::utils::State;
    use parking_lot::RwLock;
    use std::sync::Arc;

    pub trait Output: Device + Clone + 'static {
        /// The specific value the device operates on (e.g., bool, u16).
        type Value: PartialEq + Copy + Into<State> + Send + Sync;

        /// Parses the generic `State` enum into the device's specific `Value`.
        fn parse_state(&self, state: State) -> Result<Self::Value, Error>;

        /// Applies the specific `Value` to the hardware.
        fn apply_value(&mut self, value: Self::Value) -> Result<(), Error>;
        fn get_default_value(&self) -> &Self::Value;
        fn state_lock(&self) -> &RwLock<Self::Value>;
        fn animation_arc(&self) -> &Arc<Option<Animation>>;
        fn animation_arc_mut(&mut self) -> &mut Arc<Option<Animation>>;
    }
}

/// Implementation of the `Device` and `OutputDevice` traits for a concrete type implementing `sealed::Output`.
///
/// ⚠️ Necessary to work around the current limitation of `typetag`, which does not support
/// deserialization of generic implementations. This macro preserves the
/// generic behavior in `sealed::Output` while enabling (de)serialization with `typetag`.
///
/// Usage:
///
/// ```ignore
/// use hermes_five::devices::Led;
/// use hermes_five::generate_output_device_boilerplate;
///
/// generate_output_device_boilerplate!(Led);
/// ```
///
/// This generates:
/// - `impl Device for Led`
/// - `impl OutputDevice for Led` with full serialization support
/// - `impl Drop for Led` to drop any ongoing animation
#[macro_export]
macro_rules! generate_output_device_boilerplate {
    ($type:ty) => {
        use $crate::devices::output::sealed::Output;

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl $crate::devices::Device for $type {}

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl $crate::devices::OutputDevice for $type {
            fn get_state(&self) -> $crate::utils::State {
                (*self.state_lock().read()).into()
            }

            fn set_state(&mut self, state: $crate::utils::State) -> Result<$crate::utils::State, $crate::errors::Error> {
                let value = self.parse_state(state)?;

                if *self.state_lock().read() == value {
                    return Ok(value.into());
                }

                self.apply_value(value)?;
                *self.state_lock().write() = value;

                Ok(value.into())
            }

            fn get_default(&self) -> $crate::utils::State {
                (*self.get_default_value()).into()
            }

            fn animate<S: Into<$crate::utils::State>>(&mut self, state: S, duration: u64, transition: $crate::animations::Easing) {
                self.stop();
                let mut animation = $crate::animations::Animation::from(
                    $crate::animations::Track::new(self.clone())
                        .with_keyframe($crate::animations::Keyframe::new(state, 0, duration).set_transition(transition)),
                );
                animation.play();
                *self.animation_arc_mut() = std::sync::Arc::new(Some(animation));
            }

            fn is_busy(&self) -> bool {
                self.animation_arc().is_some()
            }

            fn stop(&mut self) {
                if let Some(animation) = std::sync::Arc::get_mut(self.animation_arc_mut()).and_then(Option::as_mut) {
                    animation.stop();
                }
                *self.animation_arc_mut() = std::sync::Arc::new(None);
            }
        }

        impl Drop for $type {
            fn drop(&mut self) {
                self.stop(); // Nettoie l'animation si elle est active
            }
        }
    };
}

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
