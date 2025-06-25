use crate::errors::Error;
use dyn_clone::DynClone;
use std::fmt::{Debug, Display};

mod serial;
mod wifi;

use crate::utils::private::TraitToAny;
pub use serial::Serial;
pub use wifi::WiFi;

dyn_clone::clone_trait_object!(IoTransport);

#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait IoTransport: Debug + Display + DynClone + Send + Sync + TraitToAny {
    /// Opens communication (in a blocking way) using the transport layer.
    ///
    /// # Notes
    ///  The method is sync and may block until the connection is established.
    fn open(&self) -> Result<(), Error>;

    /// Gracefully shuts down the transport layer.
    fn close(&self) -> Result<(), Error>;

    /// Write bytes to the internal connection. For more details see [`std::io::Write::write`].
    ///
    /// # Notes
    /// This function blocks until the write operation is complete. Ensure proper error handling in calling code.
    fn write(&self, buf: &[u8]) -> Result<(), Error>;

    /// Reads from the internal connection. For more details see [`std::io::Read::read_exact`].
    ///
    /// # Notes
    /// This function blocks until the buffer is filled or an error occurs. Ensure proper error handling in calling code.
    fn read_exact(&self, buf: &mut [u8]) -> Result<(), Error>;
}
