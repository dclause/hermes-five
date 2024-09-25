use std::fmt::{Debug, Display};

use dyn_clone::DynClone;

pub use crate::devices::led::Led;
pub use crate::devices::servo::Servo;
pub use crate::devices::servo::ServoType;
use crate::errors::Error;
use crate::utils::{Easing, State};
use crate::utils::scale::Scalable;

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
pub trait Actuator: Device {
    fn animate<S: Into<State>>(&mut self, state: S, duration: u64, transition: Easing)
    where
        Self: Sized;
    fn stop(&self);
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
            _ => target,
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
dyn_clone::clone_trait_object!(Actuator);

/// A trait for devices that can sense or measure data.
///
/// This trait extends `Device` and is intended for sensors that require the same capabilities
/// as devices, including debugging, cloning, and concurrency support.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Sensor: Device {}
dyn_clone::clone_trait_object!(Sensor);

#[cfg(feature = "serde")]
pub mod arc_rwlock_serde {
    use std::sync::Arc;

    use parking_lot::RwLock;
    use serde::{Deserialize, Serialize};
    use serde::de::Deserializer;
    use serde::ser::Serializer;

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
}
