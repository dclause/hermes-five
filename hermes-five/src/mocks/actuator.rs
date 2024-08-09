use crate::Board;
use crate::devices::{Actuator, Device};
use crate::errors::Error;

/// Mock [`Actuator`] for testing purposes.
#[derive(Clone, Debug)]
pub struct MockActuator {
    state: u16,
}

impl MockActuator {
    pub(crate) fn new(state: u16) -> Self {
        Self { state }
    }
}

impl Device for MockActuator {
    fn set_board(&mut self, _: &Board) {}
}

impl Actuator for MockActuator {
    fn _set_state(&mut self, state: u16) -> Result<(), Error> {
        self.state = state;
        Ok(())
    }

    /// Retrieves the actuator default (or neutral) state.
    fn get_default(&self) -> u16 {
        0
    }

    fn get_state(&self) -> u16 {
        self.state
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
