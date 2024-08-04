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

impl Device for MockActuator {}
impl Actuator for MockActuator {
    fn set_state(&mut self, state: u16) -> Result<(), Error> {
        self.state = state;
        Ok(())
    }

    fn get_state(&self) -> u16 {
        self.state
    }
}
