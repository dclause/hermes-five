//! Defines devices of various [`Input`] / [`OutputDevice`] kinds (led, servo, button, sensor, etc.) to be controlled.

mod input;
mod output;

// Input devices re-exports
pub use crate::devices::input::analog::AnalogInput;
pub use crate::devices::input::button::Button;
pub use crate::devices::input::digital::DigitalInput;
pub use crate::devices::input::{Input, InputEvent};
// Output devices re-exports
pub use crate::devices::output::sealed;
pub use crate::devices::output::digital::DigitalOutput;
pub use crate::devices::output::led::Led;
pub use crate::devices::output::pwm::PwmOutput;
pub use crate::devices::output::servo::Servo;
pub use crate::devices::output::servo::ServoType;
pub use crate::devices::output::OutputDevice;

use dyn_clone::DynClone;
use std::fmt::{Debug, Display};

/// A trait for devices that can be debugged, cloned, and used in concurrent contexts.
/// `Device` are one of the entities defined in Hermes-Five project: it represents a physical
/// device that is plugged to and can be controlled by a [`Board`](crate::hardware::Board). `Device`s come in two flavor:
/// - `Actuator`: device that can act on the world
/// - `Sensor`: device that can sense or measure data from the world
///
/// Implementors of this trait are required to be `Debug`, `DynClone`, `Send`, and `Sync`.
/// This ensures that devices can be cloned and used safely in multithreaded and async environments.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Device: Debug + Display + DynClone + Send + Sync {}
dyn_clone::clone_trait_object!(Device);