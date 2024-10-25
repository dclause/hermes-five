use crate::errors::Error;
use crate::io::private::TraitToAny;
use dyn_clone::DynClone;
use std::fmt::{Debug, Display};
use std::time::Duration;

pub mod serial;

pub(crate) mod private {
    use std::any::Any;

    pub trait TraitToAny: 'static {
        fn as_any(&self) -> &dyn Any;
    }

    impl<T: 'static> TraitToAny for T {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}

dyn_clone::clone_trait_object!(IoTransport);

#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait IoTransport: Debug + Display + DynClone + Send + Sync + TraitToAny {
    /// Opens communication (in a blocking way) using the transport layer.
    ///
    /// # Notes
    ///  The method is sync and may block until the connection is established.
    fn open(&mut self) -> Result<(), Error>;

    /// Gracefully shuts down the transport layer.
    fn close(&mut self) -> Result<(), Error>;

    /// Sets a timeout for the transport layer
    ///
    /// # Notes
    /// This function is optional and may not be supported by all transport layers.
    fn set_timeout(&mut self, duration: Duration) -> Result<(), Error>;

    /// Write bytes to the internal connection. For more details see [`std::io::Write::write`].
    ///
    /// # Notes
    /// This function blocks until the write operation is complete. Ensure proper error handling in calling code.
    fn write(&mut self, buf: &[u8]) -> Result<(), Error>;

    /// Reads from the internal connection. For more details see [`std::io::Read::read_exact`].
    ///
    /// # Notes
    /// This function blocks until the buffer is filled or an error occurs. Ensure proper error handling in calling code.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error>;
}
