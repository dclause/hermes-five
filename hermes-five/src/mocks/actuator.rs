use crate::devices::{Actuator, Device};
use crate::errors::Error;

/// Mock [`Actuator`] for testing purposes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockActuator {
    state: u16,
}

impl MockActuator {
    pub(crate) fn new(state: u16) -> Self {
        Self { state }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for MockActuator {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Actuator for MockActuator {
    fn set_state(&mut self, state: u16) -> Result<u16, Error> {
        self.state = state;
        Ok(state)
    }

    fn get_state(&self) -> u16 {
        self.state
    }

    /// Retrieves the actuator default (or neutral) state.
    fn get_default(&self) -> u16 {
        0
    }

    /// Indicates the busy status, ie if the device is running an animation.
    fn is_busy(&self) -> bool {
        false
    }
}

impl Drop for MockActuator {
    fn drop(&mut self) {
        println!("MockActuator is dropped")
    }
}
