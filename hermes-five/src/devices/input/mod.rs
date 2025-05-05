use crate::create_event_type;
use crate::devices::Device;
use crate::utils::State;

pub mod analog;
pub mod button;
pub mod digital;

/// A trait for devices that can sense or measure data: they "input" some data into the board.
///
/// This trait extends [`Device`] and is intended for sensors that require the same capabilities
/// as devices, including debugging, cloning, and concurrency support.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Input: Device {
    /// Returns  the sensor current state.
    fn get_state(&self) -> State;
}
dyn_clone::clone_trait_object!(Input);

create_event_type!(OnChangeEvent, State);
create_event_type!(OnPressEvent, ());
create_event_type!(OnReleaseEvent, ());
create_event_type!(OnHighEvent, ());
create_event_type!(OnLowEvent, ());

/// Lists all events a Input type device can emit/listen.
pub enum InputEvent {
    /// Triggered when the Input value changes.
    OnChange(OnChangeEvent),
    /// Triggered when the button is pressed.
    OnPress(()),
    /// Triggered when the button is released.
    OnRelease(()),
    /// Triggered when a value changes to HIGH.
    OnHigh(()),
    /// Triggered when a value changes to LOW.
    OnLow(()),
}