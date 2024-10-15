use std::fmt::Display;
use std::ops::{Deref, DerefMut};

use log::trace;
use parking_lot::RwLockReadGuard;

use crate::errors::Error;
use crate::protocols::{Hardware, PinModeId, Protocol};
use crate::protocols::SerialProtocol;
use crate::utils::events::{EventHandler, EventManager};
use crate::utils::task;

/// Lists all events a Board can emit/listen.
pub enum BoardEvent {
    /// Triggered when the board connexion is established and the handshake has been made.
    OnReady,
    /// Triggered when the board connexion is closed (gracefully).
    OnClose,
}

/// Convert events to string to facilitate usage with [`EventManager`].
impl Into<String> for BoardEvent {
    fn into(self) -> String {
        let event = match self {
            BoardEvent::OnReady => "ready",
            BoardEvent::OnClose => "close",
        };
        event.into()
    }
}

/// Represents a physical board where devices can be attached and control through this API.
/// The board gives access to [`Hardware`] through a communication [`Protocol`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Board {
    /// The event manager for the board.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager,
    /// The inner protocol used by this Board.
    protocol: Box<dyn Protocol>,
}

impl Default for Board {
    /// Default implementation for a board.
    /// This method creates a board with using the SerialProtocol with default settings.
    /// Note: the board will NOT be connected until the [`open`] method is called.
    ///
    /// # Example
    ///
    /// ```
    /// use hermes_five::Board;
    /// use hermes_five::protocols::SerialProtocol;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     // Following lines are all equivalent:
    ///     let board = Board::run();
    ///     let board = Board::default().open();
    ///     let board = Board::from(SerialProtocol::default()).open();
    /// }
    /// ```
    fn default() -> Self {
        Self::from(SerialProtocol::default())
    }
}

impl Board {
    /// Create and run a default board (using default protocol).
    ///
    /// # Example
    /// ```
    /// use hermes_five::Board;
    /// use hermes_five::protocols::SerialProtocol;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     // Following lines are all equivalent:
    ///     let board = Board::run();
    ///     let board = Board::default().open();
    ///     let board = Board::from(SerialProtocol::default()).open();
    /// }
    /// ```
    pub fn run() -> Self {
        Self::default().open()
    }

    /// Creates a board from a given protocol.
    ///
    /// # Example
    /// ```
    /// use hermes_five::Board;
    /// use hermes_five::protocols::SerialProtocol;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::from(SerialProtocol::new("COM4")).open();
    /// }
    /// ```
    pub fn from<P: Protocol + 'static>(protocol: P) -> Self {
        Self {
            events: EventManager::default(),
            protocol: Box::new(protocol),
        }
    }

    /// Retrieves the protocol used.
    ///
    /// NOTE: this is private to the crate since board already gives access to protocol methods via Deref.
    /// This method is only used internally in all [`Device::new()`] methods to clone the protocol into the
    /// device.
    pub fn get_protocol(&self) -> Box<dyn Protocol> {
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
    /// use hermes_five::Board;
    /// use hermes_five::BoardEvent;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///     // Is equivalent to:
    ///     let board = Board::default().open();
    ///     // Register something to do when the board is connected.
    ///     board.on(BoardEvent::OnReady, |_: Board| async move {
    ///         // Something to do when connected.
    ///         Ok(())
    ///     });
    ///     // code here will be executed right away, before the board is actually connected.
    /// }
    /// ```
    pub fn open(self) -> Self {
        let events_clone = self.events.clone();
        let callback_board = self.clone();
        task::run(async move {
            let board = callback_board.blocking_open()?;
            events_clone.emit(BoardEvent::OnReady, board);
            Ok(())
        })
        .expect("Task failed");

        self
    }

    /// Blocking version of [`Self::open()`] method.
    pub fn blocking_open(mut self) -> Result<Self, Error> {
        self.protocol.open()?;
        trace!("Board is ready: {:#?}", self.get_hardware());
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
    /// use hermes_five::{Board, pause};
    /// use hermes_five::BoardEvent;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    /// let board = Board::run();
    ///     board.on(BoardEvent::OnReady, |board: Board| async move {
    ///         // Something to do when connected.
    ///         pause!(3000);
    ///         board.close();
    ///         Ok(())
    ///     });
    ///     board.on(BoardEvent::OnClose, |_: Board| async move {
    ///         // Something to do when connection closes.
    ///         Ok(())
    ///     });
    /// }
    /// ```
    ///
    pub fn close(self) -> Self {
        let events = self.events.clone();
        let callback_board = self.clone();
        task::run(async move {
            let board = callback_board.blocking_close()?;
            events.emit(BoardEvent::OnClose, board);
            Ok(())
        })
        .expect("Task failed");
        self
    }

    /// Blocking version of [`Self::close()`] method.
    pub fn blocking_close(mut self) -> Result<Self, Error> {
        // Detach all pins.
        let pins = self.get_hardware().pins.clone();
        for (id, _) in pins {
            let _ = self.set_pin_mode(id, PinModeId::OUTPUT);
        }
        self.protocol.close()?;
        trace!("Board is closed");
        Ok(self)
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
    /// use hermes_five::Board;
    /// use hermes_five::BoardEvent;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///     board.on(BoardEvent::OnReady, |_: Board| async move {
    ///         // Here, you know the board to be connected and ready to receive data.
    ///         Ok(())
    ///     });
    /// }
    /// ```
    pub fn on<S, F, T, Fut>(&self, event: S, callback: F) -> EventHandler
    where
        S: Into<String>,
        T: 'static + Send + Sync + Clone,
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), Error>> + Send + 'static,
    {
        self.events.on(event, callback)
    }

    /// Easy access to hardware through the board.
    ///
    /// # Example
    /// ```
    /// use hermes_five::Board;
    /// use hermes_five::BoardEvent;
    /// use hermes_five::protocols::PinModeId;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///
    ///     board.on(BoardEvent::OnReady, |mut board: Board| async move {
    ///         println!("Board connected: {}", board);
    ///         println!("Pins {:#?}", board.get_hardware().pins);
    ///         Ok(())
    ///     });
    /// }
    pub fn get_hardware(&self) -> RwLockReadGuard<Hardware> {
        self.protocol.get_hardware().read()
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
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use crate::mocks::protocol::MockProtocol;
    use crate::pause;
    use crate::protocols::Message;

    use super::*;

    #[test]
    fn test_board_default() {
        // Default board can be created.
        let board = Board::default();
        assert_eq!(
            board.protocol.get_protocol_name(),
            "SerialProtocol",
            "Default board uses the default protocol"
        );
    }

    #[test]
    fn test_board_from() {
        // Custom protocol can be used.
        let board = Board::from(MockProtocol::default());
        assert_eq!(
            board.protocol.get_protocol_name(),
            "MockProtocol",
            "Board can be created with a custom protocol"
        );
    }

    #[hermes_macros::test]
    async fn test_board_open() {
        let mut protocol = MockProtocol::default();
        protocol.index = 10;
        // Result for query firmware
        protocol.buf[10..15].copy_from_slice(&[0xF0, 0x79, 0x01, 0x0C, 0xF7]);
        // Result for report capabilities
        protocol.buf[15..26].copy_from_slice(&[
            0xF0, 0x6C, 0x00, 0x08, 0x7F, 0x00, 0x08, 0x01, 0x08, 0x7F, 0xF7,
        ]);
        // Result for analog mapping
        protocol.buf[26..32].copy_from_slice(&[0xF0, 0x6A, 0x7F, 0x7F, 0x7F, 0xF7]);

        let flag = Arc::new(AtomicBool::new(false));
        let moved_flag = flag.clone();
        let board = Board::from(protocol).open();
        board.on(BoardEvent::OnReady, move |board: Board| {
            let captured_flag = moved_flag.clone();
            async move {
                captured_flag.store(true, Ordering::SeqCst);
                assert!(board.is_connected());
                Ok(())
            }
        });
        pause!(500);
        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_board_blocking_open() {
        let mut protocol = MockProtocol::default();
        protocol.index = 10;
        // Result for query firmware
        protocol.buf[10..15].copy_from_slice(&[0xF0, 0x79, 0x01, 0x0C, 0xF7]);
        // Result for report capabilities
        protocol.buf[15..26].copy_from_slice(&[
            0xF0, 0x6C, 0x00, 0x08, 0x7F, 0x00, 0x08, 0x01, 0x08, 0x7F, 0xF7,
        ]);
        // Result for analog mapping
        protocol.buf[26..32].copy_from_slice(&[0xF0, 0x6A, 0x7F, 0x7F, 0x7F, 0xF7]);

        let board = Board::from(protocol).blocking_open().unwrap();
        assert!(board.is_connected());
    }

    #[hermes_macros::test]
    async fn test_board_close() {
        let flag = Arc::new(AtomicBool::new(false));
        let moved_flag = flag.clone();

        let board = Board::from(MockProtocol::default()).open().close();
        board.on(BoardEvent::OnClose, move |board: Board| {
            let captured_flag = moved_flag.clone();
            async move {
                captured_flag.store(true, Ordering::SeqCst);
                assert!(!board.is_connected());
                Ok(())
            }
        });

        pause!(1000);
        assert!(flag.load(Ordering::SeqCst));
        assert!(!board.is_connected());
    }

    #[hermes_macros::test]
    fn test_board_run() {
        let board = Board::run();
        assert_eq!(board.protocol.get_protocol_name(), "SerialProtocol");
    }

    #[test]
    fn test_board_get_hardware() {
        let board = Board::from(MockProtocol::default());
        assert_eq!(board.get_hardware().protocol_version, "fake.1.0");
    }

    #[test]
    fn test_board_display() {
        let board = Board::from(MockProtocol::default());
        let output = format!("{}", board);
        assert_eq!(output, "Board (firmware=Fake protocol, version=fake.2.3, protocol=MockProtocol, connection=\"()\")");
    }

    #[test]
    fn test_board_deref() {
        let mut protocol = MockProtocol::default();
        protocol.buf[..3].copy_from_slice(&[0xF9, 0x01, 0x19]);
        let mut board = Board::from(protocol);
        assert_eq!(board.get_protocol().get_protocol_name(), "MockProtocol");
        assert_eq!(board.get_protocol_name(), "MockProtocol");

        let result = board.read_and_decode();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Message::ReportProtocolVersion);
        assert_eq!(board.get_hardware().protocol_version, "1.25");
    }
}
