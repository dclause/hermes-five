use std::time::Duration;

pub use crate::devices::led::Led;
pub use crate::devices::servo::Servo;
use crate::errors::Error;
use crate::misc::{Easing, State};

mod led;
mod servo;

pub trait Device: Clone {}

pub trait Actuator: Device {
    /// Set the actuator to the target step.
    fn update(&mut self, target: State) -> Result<&Self, Error>;

    /// Animate the actuator to the target step.
    ///
    /// # Parameters
    /// - `target`: the target state to reach.
    /// - `duration`: the duration taken to reach it.
    /// - `easing`: an easing method to be applied over the target.
    fn animate(
        &mut self,
        target: State,
        duration: Duration,
        easing: Easing,
    ) -> Result<&Self, Error>;
}
pub trait Sensor: Device {}
