use crate::errors::Error;
use crate::errors::ProtocolError::NotInitialized;
use crate::io::IoTransport;
use parking_lot::Mutex;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

/// Represents an [`IoTransport`] layer based on a WiFi connection.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WiFi {
    /// The connection IP:port address.
    address: String,
    /// A Read/Write io object.
    #[cfg_attr(feature = "serde", serde(skip))]
    stream: Arc<Mutex<Option<TcpStream>>>,
}

#[cfg(not(tarpaulin_include))]
impl WiFi {
    /// Constructs a new `WiFi` transport layer instance for communication through the specified IP:port.
    ///
    /// # Example
    /// ```
    /// use hermes_five::hardware::Board;
    /// use hermes_five::io::RemoteIo;
    /// use hermes_five::io::WiFi;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let protocol = RemoteIo::from(WiFi::new("192.168.1.186:3030"));
    ///     let board = Board::new(protocol).open();
    /// }
    /// ```
    pub fn new<P: Into<String>>(address: P) -> Self {
        Self {
            address: address.into(),
            stream: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns the configured address.
    pub fn get_address(&self) -> String {
        self.address.clone()
    }
}

impl Display for WiFi {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WiFi({}{})",
            self.address,
            if self.stream.lock().is_some() {
                " [*]"
            } else {
                ""
            }
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl IoTransport for WiFi {
    fn open(&mut self) -> Result<(), Error> {
        let stream = TcpStream::connect(self.address.clone())?;

        // Save the IO (required by handshake).
        self.stream = Arc::new(Mutex::new(Some(stream)));

        Ok(())
    }

    fn close(&mut self) -> Result<(), Error> {
        *self.stream.lock() = None;
        Ok(())
    }

    #[cfg(not(tarpaulin_include))]
    fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut lock = self.stream.lock();
        lock.as_mut().ok_or(NotInitialized)?.write_all(buf)?;
        Ok(())
    }

    #[cfg(not(tarpaulin_include))]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let mut lock = self.stream.lock();
        lock.as_mut().ok_or(NotInitialized)?.read_exact(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wifi_protocol() {
        let protocol = WiFi::new("192.168.1.1:3030");
        assert_eq!(protocol.address, "192.168.1.1:3030");
        assert_eq!(protocol.get_address(), "192.168.1.1:3030");
        assert!(protocol.stream.lock().is_none());
    }

    #[test]
    fn test_open_wifi_protocol() {
        // let mut protocol = get_test_successful_protocol();
        // let result = protocol.open();
        // assert!(result.is_ok());

        let mut protocol = WiFi::new("666.666.666.666:666");
        let result = protocol.open();
        assert!(result.is_err());
    }

    #[test]
    fn test_close_wifi_protocol() {
        let mut protocol = WiFi::new("666.666.666.666:666");
        let result = protocol.close();
        assert!(result.is_ok());
        assert!(protocol.stream.lock().is_none());
    }

    #[test]
    fn test_display_wifi_protocol() {
        let protocol = WiFi::new("192.168.1.1:3030");
        assert_eq!(format!("{}", protocol), "WiFi(192.168.1.1:3030)");
    }
}
