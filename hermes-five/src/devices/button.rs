use std::fmt::{Display, Formatter};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::{Board, pause};
use crate::devices::{Device, Sensor};
use crate::errors::Error;
use crate::protocols::{PinModeId, Protocol};
use crate::utils::{State, task};
use crate::utils::events::{EventHandler, EventManager};
use crate::utils::task::TaskHandler;

/// Lists all events a Button type device can emit/listen.
pub enum ButtonEvent {
    /// Triggered when the button value changes.
    OnChange,
    /// Triggered when the button is pressed.
    OnPress,
    /// Triggered when the button is released.
    OnRelease,
}

/// Convert events to string to facilitate usage with [`EventManager`].
impl Into<String> for ButtonEvent {
    fn into(self) -> String {
        let event = match self {
            ButtonEvent::OnChange => "change",
            ButtonEvent::OnPress => "press",
            ButtonEvent::OnRelease => "release",
        };
        event.into()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Button {
    // ########################################
    // # Basics
    /// The pin (id) of the board [`Board`] used to read the button value.
    pin: u16,
    /// The current Button state.
    #[cfg_attr(feature = "serde", serde(with = "crate::devices::arc_rwlock_serde"))]
    state: Arc<RwLock<bool>>,
    /// Inverts the true/false state value.
    invert: bool,
    /// Defines a PULL-UP mode button.
    pullup: bool,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol: Box<dyn Protocol>,
    /// Inner handler to the task running the button value check.
    #[cfg_attr(feature = "serde", serde(skip))]
    interval: Arc<RwLock<Option<TaskHandler>>>,
    /// The event manager for the button.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager,
}

impl Button {
    /// Creates an instance of a PULL-DOWN button attached to a given board:
    /// https://docs.arduino.cc/built-in-examples/digital/Button/
    ///
    /// - Button pressed => pin state HIGH
    /// - Button released => pin state LOW
    ///
    /// # Parameters
    /// * `board`: the board which the Button is attached to
    /// * `pin`: the pin used to read the Button value
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the Button pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the Button pin does not support INPUT mode.
    pub fn new(board: &Board, pin: u16) -> Result<Self, Error> {
        Self {
            pin,
            state: Arc::new(RwLock::new(false)),
            invert: false,
            pullup: false,
            protocol: board.get_protocol(),
            interval: Arc::new(RwLock::new(None)),
            events: Default::default(),
        }
        ._start_reactive()
    }

    /// Creates an instance of an inverted PULL-DOWN button attached to a given board:
    /// https://docs.arduino.cc/built-in-examples/digital/Button/
    ///
    /// /!\ The state value is inverted compared to HIGH/LOW electrical value of the pin.
    /// - Inverted button pressed => pin state LOW
    /// - Inverted button released => pin state HIGH
    ///
    /// # Parameters
    /// * `board`: the board which the Button is attached to
    /// * `pin`: the pin used to read the Button value
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the Button pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the Button pin does not support INPUT mode.
    pub fn new_inverted(board: &Board, pin: u16) -> Result<Self, Error> {
        Self {
            pin,
            state: Arc::new(RwLock::new(false)),
            invert: true,
            pullup: false,
            protocol: board.get_protocol(),
            interval: Arc::new(RwLock::new(None)),
            events: Default::default(),
        }
        ._start_reactive()
    }

    /// Creates an instance of a PULL-UP button attached to a given board:
    /// https://docs.arduino.cc/tutorials/generic/digital-input-pullup/
    ///
    /// - Pullup button pressed => pin state LOW
    /// - Pullup button released => pin state HIGH
    ///
    /// # Parameters
    /// * `board`: the board which the Button is attached to
    /// * `pin`: the pin used to read the Button value
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the Button pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the Button pin does not support INPUT mode.
    pub fn new_pullup(board: &Board, pin: u16) -> Result<Self, Error> {
        Self {
            pin,
            state: Arc::new(RwLock::new(true)),
            invert: false,
            pullup: true,
            protocol: board.get_protocol(),
            interval: Arc::new(RwLock::new(None)),
            events: Default::default(),
        }
        ._start_reactive()
    }

    /// Creates an instance of an inverted PULL-UP button attached to a given board:
    /// https://docs.arduino.cc/tutorials/generic/digital-input-pullup/
    ///
    /// /!\ The state value is inverted compared to HIGH/LOW electrical value of the pin
    /// (therefore equivalent to a standard pull-down button)
    /// - Inverted pullup button pressed => pin state HIGH
    /// - Inverted pullup button released => pin state LOW
    ///
    /// # Parameters
    /// * `board`: the board which the Button is attached to
    /// * `pin`: the pin used to read the Button value
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the Button pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the Button pin does not support INPUT mode.
    pub fn new_inverted_pullup(board: &Board, pin: u16) -> Result<Self, Error> {
        Self {
            pin,
            state: Arc::new(RwLock::new(true)),
            invert: true,
            pullup: true,
            protocol: board.get_protocol(),
            interval: Arc::new(RwLock::new(None)),
            events: Default::default(),
        }
        ._start_reactive()
    }

    /// Private helper method shared by constructors.
    fn _start_reactive(mut self) -> Result<Self, Error> {
        // Set pin mode to INPUT/PULLUP.
        match self.pullup {
            true => {
                self.protocol.set_pin_mode(self.pin, PinModeId::PULLUP)?;
                self.protocol
                    .get_hardware()
                    .write()
                    .get_pin_mut(self.pin)?
                    .value = 1;
            }
            false => {
                self.protocol.set_pin_mode(self.pin, PinModeId::INPUT)?;
            }
        };

        // Set reporting for this pin.
        self.protocol.report_digital_pin(self.pin, true)?;

        // Create a task to listen hardware value and emit events accordingly.
        let button_clone = self.clone();
        *self.interval.write() = Some(
            task::run(async move {
                loop {
                    let pin_value = button_clone
                        .protocol
                        .get_hardware()
                        .read()
                        .get_pin(self.pin)?
                        .value
                        != 0;
                    let state_value = button_clone.state.read().clone();
                    if pin_value != state_value {
                        *button_clone.state.write() = pin_value;

                        // Depending on logical inversion mode, pin_value is inverted.
                        match button_clone.invert {
                            false => button_clone.events.emit(ButtonEvent::OnChange, pin_value),
                            true => button_clone.events.emit(ButtonEvent::OnChange, !pin_value),
                        };

                        match button_clone.pullup {
                            true => match pin_value {
                                true => button_clone.events.emit(ButtonEvent::OnRelease, ()),
                                false => button_clone.events.emit(ButtonEvent::OnPress, ()),
                            },
                            false => match pin_value {
                                true => button_clone.events.emit(ButtonEvent::OnPress, ()),
                                false => button_clone.events.emit(ButtonEvent::OnRelease, ()),
                            },
                        };
                    }

                    // Change can only be done 10x a sec. to avoid bouncing.
                    pause!(100);
                }
                #[allow(unreachable_code)]
                Ok(())
            })
            .unwrap(),
        );

        Ok(self)
    }

    // ########################################

    /// Retrieves if the button is configured in PULL-UP mode.
    pub fn is_pullup(&self) -> bool {
        self.pullup
    }

    /// Retrieves if the logical button value is inverted.
    pub fn is_inverted(&self) -> bool {
        self.invert
    }

    // ########################################
    // Event related functions
    //
    /// Registers a callback to be executed on a given event on the Button.
    ///
    /// Available events for a button are:
    /// * `change`: Triggered when the button value changes. To use it, register though the [`Self::on()`] method.
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

impl Drop for Button {
    fn drop(&mut self) {
        if let Some(handler) = self.interval.read().as_ref() {
            handler.abort();
        }
    }
}

impl Display for Button {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Button (pin={}) [state={}, pullup={}, inverted={}]",
            self.pin,
            self.state.read(),
            self.pullup,
            self.invert
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for Button {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Sensor for Button {
    fn get_state(&self) -> State {
        match self.invert {
            false => State::from(*self.state.read()),
            true => State::from(!*self.state.read()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use crate::Board;
    // Assuming there's a mock protocol for testing
    use crate::mocks::protocol::MockProtocol;

    use super::*;

    #[hermes_macros::test]
    fn test_new_button_creation() {
        let board = Board::from(MockProtocol::default());
        let button = Button::new(&board, 4);

        assert!(button.is_ok());
        let button = button.unwrap();
        assert_eq!(button.pin, 4);
        assert!(!button.is_inverted());
        assert!(!button.is_pullup());

        drop(button);
    }

    #[hermes_macros::test]
    fn test_new_inverted_button_creation() {
        let board = Board::from(MockProtocol::default());
        let button = Button::new_inverted(&board, 4);

        assert!(button.is_ok());
        let button = button.unwrap();
        assert_eq!(button.pin, 4);
        assert!(button.is_inverted());
        assert!(!button.is_pullup());

        drop(button);
    }

    #[hermes_macros::test]
    fn test_new_pullup_button_creation() {
        let board = Board::from(MockProtocol::default());
        let button = Button::new_pullup(&board, 4);

        assert!(button.is_ok());
        let button = button.unwrap();
        assert_eq!(button.pin, 4);
        assert!(!button.is_inverted());
        assert!(button.is_pullup());

        drop(button);
    }

    #[hermes_macros::test]
    fn test_new_inverted_pullup_button_creation() {
        let board = Board::from(MockProtocol::default());
        let button = Button::new_inverted_pullup(&board, 4);

        assert!(button.is_ok());
        let button = button.unwrap();
        assert_eq!(button.pin, 4);
        assert!(button.is_inverted());
        assert!(button.is_pullup());

        drop(button);
    }

    #[hermes_macros::test]
    fn test_button_display() {
        let board = Board::from(MockProtocol::default());
        let button = Button::new(&board, 4).unwrap();

        assert_eq!(
            format!("{}", button),
            String::from("Button (pin=4) [state=false, pullup=false, inverted=false]")
        );

        drop(button);
    }

    #[hermes_macros::test]
    fn test_button_event_emission() {
        let board = Board::from(MockProtocol::default());
        let button = Button::new(&board, 4).unwrap();

        // Simulate an event handler
        let flag = Arc::new(AtomicBool::new(false));
        let moved_flag = flag.clone();
        button.on(ButtonEvent::OnChange, move |new_state: bool| {
            let captured_flag = moved_flag.clone();
            async move {
                assert!(new_state);
                captured_flag.store(new_state, Ordering::SeqCst);
                Ok(())
            }
        });

        // Simulate pin state change in the protocol
        button
            .protocol
            .get_hardware()
            .write()
            .get_pin_mut(4)
            .unwrap()
            .value = 1;

        // Ensure event is emitted
        pause!(500);
        assert!(flag.load(Ordering::SeqCst));

        drop(button);
    }

    #[hermes_macros::test]
    fn test_button_inverted_state_logic() {
        let board = Board::from(MockProtocol::default());
        let button = Button::new_inverted(&board, 4).unwrap();
        assert_eq!(button.get_state().as_bool(), true);

        button.state.write().clone_from(&true); // Simulate a pressed button
        assert_eq!(button.get_state().as_bool(), false);

        drop(button);
    }
}
