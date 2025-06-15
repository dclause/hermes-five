use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc};
use parking_lot::RwLock;
use crate::animations::Animation;
use crate::errors::Error;
use crate::generate_output_device_boilerplate;
use crate::utils::State;

/// Mock [`OutputDevice`] for testing purposes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct MockOutputDevice {
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::arc_atomic_serde"))]
    state: Arc<AtomicU16>,
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::arc_rwlock_serde"))]
    locked_state: Arc<RwLock<u16>>, // Used for serde testing
    #[cfg_attr(feature = "serde", serde(skip))]
    animation: Arc<Option<Animation>>,
}

impl MockOutputDevice {
    pub fn new(state: u16) -> Self {
        Self {
            state: Arc::new(AtomicU16::new(state)),
            locked_state: Arc::new(RwLock::new(42)),
            animation: Arc::new(None),
        }
    }

    pub fn get_locked_value(&self) -> u16 { *self.locked_state.read() }
}

impl Display for MockOutputDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockActuator [state={}]", self.state.load(Ordering::SeqCst))
    }
}

generate_output_device_boilerplate!(MockOutputDevice);
impl Output for MockOutputDevice {
    type Value = u16;

    fn parse_state(&self, state: State) -> Result<Self::Value, Error> {
        Ok(state.as_integer() as u16)
    }

    fn apply_value(&mut self, _: Self::Value) -> Result<(), Error> {
        // Nothing to do
        Ok(())
    }

    // Expose the required fields
    fn get_default_value(&self) -> Self::Value { 0 }
    fn get_value(&self) -> Self::Value { self.state.load(Ordering::SeqCst) }
    fn set_value(&self, value: Self::Value) { self.state.store(value, Ordering::SeqCst) }
    fn animation_arc(&self) -> &Arc<Option<Animation>> { &self.animation }
    fn animation_arc_mut(&mut self) -> &mut Arc<Option<Animation>> { &mut self.animation }
}
