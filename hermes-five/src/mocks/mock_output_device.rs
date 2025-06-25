use crate::errors::Error;
use crate::utils::State;
use hermes_five_macros::output_device;
use parking_lot::RwLock;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

/// Mock [`OutputDevice`] for testing purposes.
#[output_device(u16)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockOutputDevice {
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::serde_arc_atomic"))]
    state: Arc<AtomicU16>,
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::serde_arc_rwlock"))]
    locked_state: Arc<RwLock<u16>>, // Used for serde testing
}

impl MockOutputDevice {
    pub fn new(state: u16) -> Self {
        Self {
            state: Arc::new(AtomicU16::new(state)),
            locked_state: Arc::new(RwLock::new(42)),
            default: 0,
            animation: Arc::new(None),
        }
    }

    pub fn get_locked_value(&self) -> u16 {
        *self.locked_state.read()
    }

    fn parse_state(&self, state: State) -> Result<u16, Error> {
        Ok(state.as_integer() as u16)
    }

    fn apply_value(&mut self, _: u16) -> Result<(), Error> {
        // Nothing to do
        Ok(())
    }

    fn get_value(&self) -> u16 {
        self.state.load(Ordering::Relaxed)
    }
    fn set_value(&self, value: u16) {
        self.state.store(value, Ordering::Relaxed)
    }
}

impl Display for MockOutputDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MockActuator [state={}]",
            self.state.load(Ordering::Relaxed)
        )
    }
}
