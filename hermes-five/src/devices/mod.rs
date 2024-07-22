use std::fmt::Debug;

use async_trait::async_trait;
use dyn_clone::DynClone;

pub use crate::devices::led::Led;
pub use crate::devices::servo::Servo;
use crate::errors::Error;

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
pub trait Device: Debug + DynClone + Send + Sync {}
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
#[async_trait]
pub(crate) trait Actuator: Device {
    /// Internal only.
    fn set_state(&mut self, state: u16) -> Result<(), Error>;
    /// Internal only.
    fn get_state(&self) -> u16;
}
dyn_clone::clone_trait_object!(Actuator);

/// A trait for devices that can sense or measure data.
///
/// This trait extends `Device` and is intended for sensors that require the same capabilities
/// as devices, including debugging, cloning, and concurrency support.
pub trait Sensor: Device {}
dyn_clone::clone_trait_object!(Sensor);
