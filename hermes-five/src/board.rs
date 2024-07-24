use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use parking_lot::RwLockReadGuard;

use crate::errors::Error;
use crate::protocols::{Hardware, Protocol};
use crate::protocols::SerialProtocol;
use crate::utils::events::{EventHandler, EventManager};
use crate::utils::task;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Board {
    /// The event manager for the board.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager,
    /// The inner protocol used by this Board.
    protocol: Box<dyn Protocol>,
}

/// Custom clone: do not clone events.
impl Clone for Board {
    fn clone(&self) -> Self {
        Self {
            events: EventManager::default(),
            protocol: self.protocol.clone(),
        }
    }
}

impl Default for Board {
    /// Default implementation for a board.
    /// This method creates a board with using the SerialProtocol with default settings.
    /// Note: the board will NOT be connected until the `open` method is called.
    ///
    /// # Example
    /// // Following lines are all equivalent:
    /// let board = Board::run();
    /// let board = Board::default().open();
    /// let board = Board::build(SerialProtocol::default()).open();
    /// let board = Board::default().with_protocol(SerialProtocol::default()).open();
    /// ```
    fn default() -> Self {
        Self::build(SerialProtocol::default())
    }
}

impl Board {
    /// Create and run a default board (using default protocol).
    ///
    /// # Example
    /// ```
    /// // Following lines are all equivalent:
    /// let board = Board::run();
    /// let board = Board::default().open();
    /// let board = Board::build(SerialProtocol::default()).open();
    /// let board = Board::default().with_protocol(SerialProtocol::default()).open();
    /// ```
    pub fn run() -> Self {
        Self::default().open()
    }

    /// Creates a board using the given protocol.
    ///
    /// # Example
    /// ```
    /// let board = Board::build(SerialProtocol::new("COM4")).open()
    /// ```
    pub fn build<P: Protocol + 'static>(protocol: P) -> Self {
        Self {
            events: Default::default(),
            protocol: Box::new(protocol),
        }
    }

    /// Setter for board protocol.
    ///
    /// # Example
    /// ```
    /// let board = Board::default().with_protocol(SerialProtocol::new("COM4")).open();
    /// ```
    pub fn with_protocol<P: Protocol + 'static>(mut self, protocol: P) -> Self {
        self.protocol = Box::new(protocol);
        self
    }

    /// Retrieve the protocol used.
    /// This is not exposed outside since it should not be necessary thanks to Deref implementation
    /// but to clone a protocol out of the board like done in Device initialisations
    pub(crate) fn protocol(&self) -> Box<dyn Protocol> {
        self.protocol.clone()
    }

    /// Starts a board connexion procedure (using the appropriate configured protocol) in an asynchronous way.
    /// _Note 1:    you probably might not want to call this method yourself and use [`Self::run()`] instead._
    /// _Note 2:    after this method, you cannot consider the board to be connected until you receive the "ready" event._
    ///
    /// # Example
    ///
    /// Have a look at the examples/board folder more detailed examples.
    ///
    /// ```
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///     // Is equivalent to:
    ///     let board = Board::default().open();
    ///     // Register something to do when the board is connected.
    ///     board.on("ready", || async move {
    ///         // Something to do when connected.
    ///     });
    ///     // code here will be executed right away, before the board is actually connected.
    /// }
    /// ```
    ///
    pub fn open(self) -> Self {
        let events = self.events.clone();
        let mut callback_board = self.clone();
        let _ = task::run(async move {
            callback_board.protocol.open()?;
            // give it some time: some arduino (like nano) may be slow.
            tokio::time::sleep(Duration::from_millis(200)).await;
            callback_board.protocol.handshake()?;
            events.emit("ready", callback_board);
            Ok(())
        });
        self
    }

    /// Blocking version of [`Self::open()`] method.
    pub fn blocking_open(mut self) -> Result<Self, Error> {
        self.protocol.open()?;
        self.protocol.handshake()?;
        Ok(self)
    }

    /// Close a board connexion (using the appropriate configured protocol) in an asynchronous way.
    /// _Note:    after this method, you cannot consider the board to be connected until you receive the "close" event._
    ///
    /// # Example
    ///
    /// Have a look at the examples/board folder more detailed examples.
    ///
    /// ```
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///     board.on("ready", || async move {
    ///         // Something to do when connected.
    ///         hermes_five::utils::sleep(std::time::Duration::from_secs(3)).await;
    ///         board.close();
    ///     });
    ///     board.on("close", || async move {
    ///         // Something to do when connection closes.
    ///     });
    /// }
    /// ```
    ///
    pub fn close(self) -> Self {
        let events = self.events.clone();
        let mut protocol = self.protocol.clone();
        let callback_board = self.clone();
        let _ = task::run(async move {
            protocol.close()?;
            events.emit("close", callback_board);
            Ok(())
        });
        self
    }

    // ########################################
    // Event related functions

    /// Registers a callback to be executed on a given event on the board.
    ///
    /// Available events for a board are:
    /// * `ready`: Triggered when the board is connected and ready to run. To use it, register though the [`Self::on()`] method.
    /// * `exit`: Triggered when the board is disconnected. To use it, register though the [`Self::on()`] method.
    ///
    /// # Example
    ///
    /// ```
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board1 = Board::run();
    ///     board.on("ready", || async move {
    ///         // Here, you know the board to be connected and ready to receive data.
    ///     }).await;
    /// }
    /// ```
    pub async fn on<S, F, T, Fut>(&self, event: S, callback: F) -> EventHandler
    where
        S: Into<String>,
        T: 'static + Send + Sync + Clone,
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.events.on(event, callback)
    }

    // @todo describe / verify
    pub fn hardware(&self) -> RwLockReadGuard<Hardware> {
        self.protocol.hardware().read()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Board ({})", self.protocol)
    }
}

impl Deref for Board {
    type Target = Box<dyn Protocol>;

    fn deref(&self) -> &Self::Target {
        &self.protocol
    }
}

impl DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.protocol
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::mocks::protocol::MockProtocol;

    use super::*;

    #[test]
    fn test_board_create() {
        // Default board can be created.
        let board = Board::default();
        assert_eq!(
            board.protocol.get_protocol_name(),
            "SerialProtocol",
            "Default board uses the default protocol"
        );

        // Default board can be created.
        let board = Board::build(MockProtocol::default());
        assert_eq!(
            board.protocol.get_protocol_name(),
            "MockProtocol",
            "Board can be created with a custom protocol"
        );

        // Default board can be created.
        let board = Board::default().with_protocol(MockProtocol::default());
        assert_eq!(
            board.protocol.get_protocol_name(),
            "MockProtocol",
            "Board can be created with a custom protocol after default"
        );
    }
}
