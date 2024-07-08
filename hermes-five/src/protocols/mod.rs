use std::fmt::Debug;

use anyhow::Result;
use dyn_clone::DynClone;

pub mod serial;

/// Defines the trait all protocols must implements.
#[cfg_attr(feature = "storage", typetag::serde(tag = "type"))]
pub trait Protocol: DynClone + Debug + Send + Sync {
    /// Open the communication using the underlying protocol.
    fn open(&mut self) -> Result<()>;
    /// Gracefully shuts down the communication.
    fn close(&mut self) -> Result<()>;
}

// Makes a Box<dyn Protocol> clone (used for Board cloning).
dyn_clone::clone_trait_object!(Protocol);
