use std::fmt::{Display, Formatter};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::animations::Easing;
use crate::devices::{Device, Output};
use crate::errors::Error;
use crate::utils::State;

/// Mock [`Output`] for testing purposes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockOutputDevice {
    state: u16,
    #[cfg_attr(feature = "serde", serde(with = "crate::devices::arc_rwlock_serde"))]
    lock: Arc<RwLock<u16>>,
}

impl MockOutputDevice {
    pub fn new(state: u16) -> Self {
        Self {
            state,
            lock: Arc::new(RwLock::new(42)),
        }
    }

    pub fn get_locked_value(&self) -> u16 {
        self.lock.read().clone()
    }
}

impl Display for MockOutputDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockActuator [state={}]", self.state)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for MockOutputDevice {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Output for MockOutputDevice {
    fn get_state(&self) -> State {
        self.state.into()
    }

    fn set_state(&mut self, state: State) -> Result<State, Error> {
        self.state = state.as_integer() as u16;
        Ok(state)
    }

    /// Returns  the actuator default (or neutral) state.
    fn get_default(&self) -> State {
        0.into()
    }

    fn animate<S: Into<State>>(&mut self, _: S, _: u64, _: Easing) {
        todo!()
    }

    /// Indicates the busy status, ie if the device is running an animation.
    fn is_busy(&self) -> bool {
        false
    }

    fn stop(&mut self) {}
}
