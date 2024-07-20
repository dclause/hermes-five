use async_trait::async_trait;
use dyn_clone::DynClone;

pub use crate::devices::led::Led;
pub use crate::devices::servo::Servo;
use crate::errors::Error;
use crate::utils::Easing;

mod led;
mod servo;

pub trait Device: DynClone + Send + Sync {}
dyn_clone::clone_trait_object!(Device);

/// Represents a device that is able to act on the world.
#[async_trait]
pub trait Actuator: Device {
    /// Update the actuator according to current internal state.
    fn update(&mut self) -> Result<(), Error>;

    /// Animate the actuator to the target step.
    ///
    /// # Parameters
    /// - `target`: the target state to reach.
    /// - `duration`: the duration taken to reach it.
    /// - `easing`: an easing method to be applied over the target.
    async fn animate(&mut self, target: u16, duration: u32, easing: Easing);
}
dyn_clone::clone_trait_object!(Actuator);

pub trait Sensor: Device {}
