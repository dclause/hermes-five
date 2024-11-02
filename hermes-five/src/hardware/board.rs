use crate::errors::Error;
use crate::hardware::Hardware;
use crate::io::{IoData, IoTransport, RemoteIo, IO};
use crate::io::{IoProtocol, PinModeId};
use crate::utils::{task, Range};
use crate::utils::{EventHandler, EventManager};
use parking_lot::RwLock;
use std::fmt::Display;
use std::sync::Arc;

/// Lists all events a Board can emit/listen.
pub enum BoardEvent {
    /// Triggered when the board connexion is established and the handshake has been made.
    OnReady,
    /// Triggered when the board connexion is closed (gracefully).
    OnClose,
}

/// Convert events to string to facilitate usage with [`EventManager`].
impl From<BoardEvent> for String {
    fn from(value: BoardEvent) -> Self {
        let event = match value {
            BoardEvent::OnReady => "ready",
            BoardEvent::OnClose => "close",
        };
        event.into()
    }
}

/// Represents a physical board (Arduino most-likely) where your [`crate::devices::Device`] can be attached and controlled through this API.
/// The board gives access to [`IoData`] through a communication [`IoProtocol`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Board {
    /// The event manager for the board.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager,
    /// The inner protocol used by this Board.
    protocol: Box<dyn IoProtocol>,
}

impl Default for Board {
    /// Default implementation for a board.
    ///
    /// This method creates a board using the default [`RemoteIo`] protocol with [`Serial`](crate::io::Serial) transport layer.
    /// The port will be auto-detected as the first available serial port matching a board.
    ///
    /// **_/!\ The board will NOT be connected until the [`Board::open`] method is called._**
    ///
    /// # Example
    ///
    /// ```
    /// use hermes_five::hardware::Board;
    /// use hermes_five::io::RemoteIo;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     // Following lines are all equivalent:
    ///     let board = Board::run();
    ///     let board = Board::default().open();
    ///     let board = Board::new(RemoteIo::default()).open();
    /// }
    /// ```
    fn default() -> Self {
        Self::new(RemoteIo::default())
    }
}

/// Creates a board using the given transport layer with the RemoteIo protocol.
///
/// # Example
/// ```
/// use hermes_five::hardware::Board;
/// use hermes_five::io::RemoteIo;
/// use hermes_five::io::Serial;
///
/// #[hermes_five::runtime]
/// async fn main() {
///     let board = Board::from(Serial::new("/dev/ttyUSB0")).open();
/// }
/// ```
impl<T: IoTransport> From<T> for Board {
    fn from(transport: T) -> Self {
        Self {
            events: Default::default(),
            protocol: Box::new(RemoteIo::from(transport)),
        }
    }
}

impl Board {
    /// Creates and open a default board (using default protocol).
    ///
    /// This method creates a board using the default [`RemoteIo`] protocol with [`Serial`](crate::io::Serial) transport layer.
    /// The port will be auto-detected as the first available serial port matching a board.
    ///
    /// # Example
    /// ```
    /// use hermes_five::hardware::Board;
    /// use hermes_five::io::RemoteIo;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     // Following lines are all equivalent:
    ///     let board = Board::run();
    ///     let board = Board::default().open();
    ///     let board = Board::new(RemoteIo::default()).open();
    /// }
    /// ```
    pub fn run() -> Self {
        Self::default().open()
    }

    /// Creates a board using a given protocol.
    ///
    /// # Example
    /// ```
    /// use hermes_five::hardware::Board;
    /// use hermes_five::io::RemoteIo;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::new(RemoteIo::new("COM4")).open();
    /// }
    /// ```
    pub fn new<P: IoProtocol + 'static>(protocol: P) -> Self {
        Self {
            events: EventManager::default(),
            protocol: Box::new(protocol),
        }
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
    /// use hermes_five::hardware::{Board, BoardEvent};
    /// use hermes_five::io::IO;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///     // Is equivalent to:
    ///     let mut board = Board::default().open();
    ///
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

    /// Close a board connexion (using the appropriate configured protocol) in an asynchronous way.
    /// _Note:    after this method, you cannot consider the board to be connected until you receive the "close" event._
    ///
    /// # Example
    ///
    /// Have a look at the examples/board folder more detailed examples.
    ///
    /// ```
    /// use hermes_five::pause;
    /// use hermes_five::hardware::{Board, BoardEvent};
    /// use hermes_five::io::IO;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///     board.on(BoardEvent::OnReady, |mut board: Board| async move {
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

    /// Blocking version of [`Self::open()`] method.
    pub fn blocking_open(mut self) -> Result<Self, Error> {
        self.protocol.open()?;
        // trace!"Board is ready: {:#?}", self.get_io());
        Ok(self)
    }

    /// Blocking version of [`Self::close()`] method.
    pub fn blocking_close(mut self) -> Result<Self, Error> {
        // Detach all pins.
        let pins: Vec<u8> = self.get_io().read().pins.keys().copied().collect();
        for id in pins {
            let _ = self.set_pin_mode(id, PinModeId::OUTPUT);
        }
        self.protocol.close()?;
        // trace!"Board is closed");
        Ok(self)
    }

    /// Registers a callback to be executed on a given event.
    ///
    /// Available events for a board are defined by the enum: [`BoardEvent`]:
    /// - **`OnRead` | `ready`:** Triggered when the board is connected and ready to run.    
    ///    _The callback must receive the following parameter: `|_: Board| { ... }`_
    /// - **`OnClose` | `close`:** Triggered when the board is disconnected.        
    ///    _The callback must receive the following parameter: `|_: Board| { ... }`_
    ///
    /// # Example
    ///
    /// ```
    /// use hermes_five::hardware::{Board, BoardEvent};
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
}

impl Hardware for Board {
    /// Returns  the protocol used.
    ///
    /// NOTE: this is private to the crate since board already gives access to protocol methods via Deref.
    /// This method is only used internally in all [`Device::new()`] methods to clone the protocol into the
    /// device.
    fn get_protocol(&self) -> Box<dyn IoProtocol> {
        self.protocol.clone()
    }
}

// Note: no need to test cover: those are simple pass through only.
#[cfg(not(tarpaulin_include))]
impl IO for Board {
    /// Easy access to hardware through the board.
    ///
    /// # Example
    /// ```
    /// use hermes_five::hardware::Board;
    /// use hermes_five::hardware::BoardEvent;
    /// use hermes_five::io::IO;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///     board.on(BoardEvent::OnReady, |mut board: Board| async move {
    ///         println!("Board connected: {}", board);
    ///         println!("Pins {:#?}", board.get_io().read().pins);
    ///         Ok(())
    ///     });
    /// }
    fn get_io(&self) -> &Arc<RwLock<IoData>> {
        self.protocol.get_io()
    }

    fn is_connected(&self) -> bool {
        self.protocol.is_connected()
    }

    fn set_pin_mode(&mut self, pin: u8, mode: PinModeId) -> Result<(), Error> {
        self.protocol.set_pin_mode(pin, mode)
    }

    fn digital_write(&mut self, pin: u8, level: bool) -> Result<(), Error> {
        self.protocol.digital_write(pin, level)
    }

    fn analog_write(&mut self, pin: u8, level: u16) -> Result<(), Error> {
        self.protocol.analog_write(pin, level)
    }

    #[cfg(not(tarpaulin_include))]
    fn digital_read(&mut self, _: u8) -> Result<bool, Error> {
        unimplemented!()
    }

    #[cfg(not(tarpaulin_include))]
    fn analog_read(&mut self, _: u8) -> Result<u16, Error> {
        unimplemented!()
    }

    fn servo_config(&mut self, pin: u8, pwm_range: Range<u16>) -> Result<(), Error> {
        self.protocol.servo_config(pin, pwm_range)
    }

    fn i2c_config(&mut self, delay: u16) -> Result<(), Error> {
        self.protocol.i2c_config(delay)
    }

    fn i2c_read(&mut self, address: u8, size: u16) -> Result<(), Error> {
        self.protocol.i2c_read(address, size)
    }

    fn i2c_write(&mut self, address: u8, data: &[u16]) -> Result<(), Error> {
        self.protocol.i2c_write(address, data)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Board ({})", self.protocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Serial;
    use crate::io::IO;
    use crate::mocks::plugin_io::MockIoProtocol;
    use crate::mocks::transport_layer::MockTransportLayer;
    use crate::pause;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_board_default() {
        // Default board can be created.
        let board = Board::default();
        assert_eq!(
            board.get_protocol_name(),
            "RemoteIo",
            "Default board uses the default protocol"
        );
    }

    #[test]
    fn test_board_from() {
        // Custom protocol can be used.
        let board = Board::new(MockIoProtocol::default());
        assert_eq!(
            board.get_protocol_name(),
            "MockIoProtocol",
            "Board can be created with a custom protocol"
        );
        // Custom transport can be used.
        let board = Board::from(Serial::default());
        assert_eq!(
            board.get_protocol_name(),
            "RemoteIo",
            "Board can be created with a custom transport"
        );
    }

    #[hermes_macros::test]
    async fn test_board_open() {
        let mut transport = MockTransportLayer {
            read_index: 10,
            ..Default::default()
        };
        // Result for query firmware
        transport.read_buf[10..15].copy_from_slice(&[0xF0, 0x79, 0x01, 0x0C, 0xF7]);
        // Result for report capabilities
        transport.read_buf[15..26].copy_from_slice(&[
            0xF0, 0x6C, 0x00, 0x08, 0x7F, 0x00, 0x08, 0x01, 0x08, 0x7F, 0xF7,
        ]);
        // Result for analog mapping
        transport.read_buf[26..32].copy_from_slice(&[0xF0, 0x6A, 0x7F, 0x7F, 0x7F, 0xF7]);

        let flag = Arc::new(AtomicBool::new(false));
        let moved_flag = flag.clone();
        let board = Board::new(RemoteIo::from(transport)).open();
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
        let mut transport = MockTransportLayer {
            read_index: 10,
            ..Default::default()
        };
        // Result for query firmware
        transport.read_buf[10..15].copy_from_slice(&[0xF0, 0x79, 0x01, 0x0C, 0xF7]);
        // Result for report capabilities
        transport.read_buf[15..26].copy_from_slice(&[
            0xF0, 0x6C, 0x00, 0x08, 0x7F, 0x00, 0x08, 0x01, 0x08, 0x7F, 0xF7,
        ]);
        // Result for analog mapping
        transport.read_buf[26..32].copy_from_slice(&[0xF0, 0x6A, 0x7F, 0x7F, 0x7F, 0xF7]);

        let protocol = RemoteIo::from(transport);
        let board = Board::new(protocol).blocking_open().unwrap();
        assert!(board.is_connected());
    }

    #[hermes_macros::test]
    async fn test_board_close() {
        let flag = Arc::new(AtomicBool::new(false));
        let moved_flag = flag.clone();

        let board = Board::new(MockIoProtocol::default()).open().close();

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
        assert_eq!(board.get_protocol_name(), "RemoteIo");
        board.close();
    }

    #[test]
    fn test_board_get_hardware() {
        let board = Board::new(MockIoProtocol::default());
        assert_eq!(board.get_io().read().protocol_version, "fake.1.0");
    }

    #[test]
    fn test_board_display() {
        let board = Board::new(MockIoProtocol::default());
        let output = format!("{}", board);
        assert_eq!(
            output,
            "Board (MockIoProtocol [firmware=Fake protocol, version=fake.2.3, protocol=fake.1.0])"
        );
    }
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod serde_tests {
    use crate::hardware::{Board, Hardware};
    use crate::io::RemoteIo;
    use crate::mocks::plugin_io::MockIoProtocol;

    #[test]
    fn test_board_serialize() {
        let board = Board::new(RemoteIo::new("mock"));
        let json = serde_json::to_string(&board).unwrap();
        assert_eq!(
            json,
            r#"{"protocol":{"type":"RemoteIo","transport":{"type":"Serial","port":"mock"}}}"#
        );

        let board = Board::new(MockIoProtocol::default());
        let json = serde_json::to_string(&board).unwrap();
        assert_eq!(json, r#"{"protocol":{"type":"MockIoProtocol"}}"#);
    }

    #[test]
    fn test_board_deserialize() {
        let json =
            r#"{"protocol":{"type":"RemoteIo","transport":{"type":"Serial","port":"mock"}}}"#;
        let board: Board = serde_json::from_str(json).unwrap();
        assert_eq!(board.get_protocol_name(), "RemoteIo");

        let json = r#"{"protocol":{"type":"MockIoProtocol"}}"#;
        let board: Board = serde_json::from_str(json).unwrap();
        assert_eq!(board.get_protocol_name(), "MockIoProtocol");
    }
}
