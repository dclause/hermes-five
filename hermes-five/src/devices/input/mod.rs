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
    /// Retrieves the sensor current state.
    fn get_state(&self) -> State;
}
dyn_clone::clone_trait_object!(Input);

/// Lists all events a Input type device can emit/listen.
pub enum InputEvent {
    /// Triggered when the Input value changes.
    OnChange,
    /// Triggered when the button is pressed.
    OnPress,
    /// Triggered when the button is released.
    OnRelease,
    /// Triggered when a value changes to HIGH.
    OnHigh,
    /// Triggered when a value changes to LOW.
    OnLow,
}

/// Convert events to string to facilitate usage with [`EventManager`].
impl Into<String> for InputEvent {
    fn into(self) -> String {
        let event = match self {
            InputEvent::OnChange => "change",
            InputEvent::OnPress => "press",
            InputEvent::OnRelease => "release",
            InputEvent::OnHigh => "high",
            InputEvent::OnLow => "low",
        };
        event.into()
    }
}
