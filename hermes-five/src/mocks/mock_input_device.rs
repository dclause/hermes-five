use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

use crate::devices::{Device, Input};
use crate::utils::State;

/// Mock [`Input`] for testing purposes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockInputDevice {
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::arc_atomic_serde"))]
    state: Arc<AtomicU16>,
}

impl MockInputDevice {
    pub fn new(state: u16) -> Self {
        Self {
            state: Arc::new(AtomicU16::new(state)),
        }
    }
}

impl Display for MockInputDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockActuator [state={}]", self.state.load(Ordering::SeqCst))
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for MockInputDevice {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Input for MockInputDevice {
    fn get_state(&self) -> State {
        self.state.load(Ordering::SeqCst).into()
    }
}
