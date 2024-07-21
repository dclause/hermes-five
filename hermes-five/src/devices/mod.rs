use std::fmt::Debug;

use async_trait::async_trait;
use dyn_clone::DynClone;

pub use crate::devices::led::Led;
pub use crate::devices::servo::Servo;
use crate::errors::Error;

mod led;
mod servo;

pub trait Device: Debug + DynClone + Send + Sync {}
dyn_clone::clone_trait_object!(Device);

/// Represents a device that is able to act on the world.
#[async_trait]
pub(crate) trait Actuator: Device {
    /// Set the actuator internal state and update it.
    fn set_state(&mut self, state: f64) -> Result<(), Error>;
    /// Internal only (used by [`Animation`]).
    fn get_state(&self) -> u16;
}
dyn_clone::clone_trait_object!(Actuator);

pub trait Sensor: Device {}
