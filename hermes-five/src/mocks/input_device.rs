use std::fmt::{Display, Formatter};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::devices::{Device, Input};
use crate::utils::State;

/// Mock [`Input`] for testing purposes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockInputDevice {
    state: u16,
    #[cfg_attr(feature = "serde", serde(with = "crate::devices::arc_rwlock_serde"))]
    lock: Arc<RwLock<u16>>,
}

impl MockInputDevice {
    pub fn new(state: u16) -> Self {
        Self {
            state,
            lock: Arc::new(RwLock::new(42)),
        }
    }

    pub fn get_locked_value(&self) -> u16 {
        *self.lock.read()
    }
}

impl Display for MockInputDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockActuator [state={}]", self.state)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for MockInputDevice {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Input for MockInputDevice {
    fn get_state(&self) -> State {
        self.state.into()
    }
}
