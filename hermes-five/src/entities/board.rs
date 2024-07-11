use std::fmt::Display;
use std::panic::UnwindSafe;

use crate::protocols::{Error, Protocol};
use crate::protocols::serial::SerialProtocol;
use crate::utils::events::{EventHandler, EventManager};
use crate::utils::task;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Board {
    /// The event manager for the board.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager,
    /// The communication protocol used by this board.
    // @todo remove 'pub' and add pass through
    pub protocol: Box<dyn Protocol>,
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
    /// let board = Board::run().await;
    /// let board = Board::default().open().await;
    /// let board = Board::build(SerialProtocol::default()).open().await;
    /// let board = Board::default().with_protocol(SerialProtocol::default()).open().await;
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
    /// let board = Board::run().await;
    /// let board = Board::default().open().await;
    /// let board = Board::build(SerialProtocol::default()).open().await;
    /// let board = Board::default().with_protocol(SerialProtocol::default()).open().await;
    /// ```
    pub async fn run() -> Self {
        Self::default().open().await
    }

    /// Creates a board using the given protocol.
    ///
    /// # Example
    /// ```
    /// let board = Board::build(SerialProtocol::new("COM4")).open().await
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
    /// let board = Board::default().with_protocol(SerialProtocol::new("COM4")).open().await;
    /// ```
    pub fn with_protocol<P: Protocol + 'static>(mut self, protocol: P) -> Self {
        self.protocol = Box::new(protocol);
        self
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
    ///     let board = Board::run().await;
    ///     // Is equivalent to:
    ///     let board = Board::default().open().await;
    ///     // Register something to do when the board is connected.
    ///     board.on("ready", || async move {
    ///         // Something to do when connected.
    ///     });
    ///     // code here will be executed right away, before the board is actually connected.
    /// }
    /// ```
    ///
    pub async fn open(self) -> Self {
        let events = self.events.clone();
        let mut callback_board = self.clone();
        task::run(async move {
            callback_board.protocol.open()?;
            callback_board.protocol.handshake()?;
            events.emit("ready", callback_board).await;
            Ok(())
        })
        .await;
        self
    }

    /// Blocking version of [`Self::open()`] method.
    pub fn blocking_open(mut self) -> Result<Self, Error> {
        self.protocol.open()?;
        self.protocol.handshake()?;
        println!("Board connected: {}", self);
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
    ///     let board = Board::run().await;
    ///     board.on("ready", || async move {
    ///         // Something to do when connected.
    ///         hermes_five::utils::sleep(std::time::Duration::from_secs(3)).await;
    ///         board.close().await;
    ///     });
    ///     board.on("close", || async move {
    ///         // Something to do when connection closes.
    ///     });
    /// }
    /// ```
    ///
    pub async fn close(self) -> Self {
        let events = self.events.clone();
        let mut protocol = self.protocol.clone();
        let callback_board = self.clone();
        task::run(async move {
            protocol.close()?;
            events.emit("close", callback_board).await;
            Ok(())
        })
        .await;
        self
    }

    // ########################################
    // Protocol related

    // @todo here
    // /// Get the firmware name.
    // fn firmware_name(&mut self) -> &String;
    // /// Get the firmware version.
    // fn firmware_version(&mut self) -> &String;
    //
    // /// Get pins that the board has access to.
    // fn pins(&mut self) -> &Vec<Pin>;
    // /// Get the current Firmata protocol version.
    // fn protocol_version(&mut self) -> &String;

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
    ///     let board1 = Board::run().await;
    ///     board.on("ready", || async move {
    ///         // Here, you know the board to be connected and ready to receive data.
    ///     }).await;
    /// }
    /// ```
    pub async fn on<S, F, T, Fut>(&self, event: S, callback: F) -> EventHandler
    where
        S: Into<String>,
        T: 'static + Send + Sync + Clone,
        F: FnMut(T) -> Fut + Send + 'static + UnwindSafe,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.events.on(event, callback).await
    }
}

// @todo display the firmware from here, hence enough dervice(Debug) on protocol.
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Board ({})", self.protocol)
    }
}
