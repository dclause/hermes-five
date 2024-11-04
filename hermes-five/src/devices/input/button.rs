use std::fmt::{Display, Formatter};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::devices::{Device, Input, InputEvent};
use crate::errors::Error;
use crate::hardware::Hardware;
use crate::io::{IoProtocol, PinIdOrName, PinModeId};
use crate::pause;
use crate::utils::{task, EventHandler, EventManager, State, TaskHandler};

/// Represents a simple push button as an input of the board.
/// <https://docs.arduino.cc/built-in-examples/digital/Button>
///
/// This structure is very similar to [`DigitalInput`](crate::devices::DigitalInput) but exposes convenience methods to handle two sorts of buttons:
/// - pull-down: when the button is configured with a pin to ground by default (ie button press => pin becomes high)
/// - pull-up: when the button is configured with a pin to +Vin by default (ie button press => pin becomes low)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Button {
    // ########################################
    // # Basics
    /// The pin (id) of the [`Board`] used to read the button value.
    pin: u8,
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
    protocol: Box<dyn IoProtocol>,
    /// Inner handler to the task running the button value check.
    #[cfg_attr(feature = "serde", serde(skip))]
    handler: Arc<RwLock<Option<TaskHandler>>>,
    /// The event manager for the button.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager,
}

impl Button {
    /// Creates an instance of a PULL-DOWN button attached to a given board:
    /// <https://docs.arduino.cc/built-in-examples/digital/Button/>
    ///
    /// - Button pressed => pin state HIGH
    /// - Button released => pin state LOW
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the Button pin does not exist for this board.
    /// * `IncompatiblePin`: this function will bail an error if the Button pin does not support INPUT mode.
    pub fn new<T: Into<PinIdOrName>>(board: &dyn Hardware, pin: T) -> Result<Self, Error> {
        Self {
            pin: 0,
            state: Arc::new(RwLock::new(false)),
            invert: false,
            pullup: false,
            protocol: board.get_protocol(),
            handler: Arc::new(RwLock::new(None)),
            events: Default::default(),
        }
        .start_with(board, pin)
    }

    /// Creates an instance of an inverted PULL-DOWN button attached to a given board:
    /// <https://docs.arduino.cc/built-in-examples/digital/Button/>
    ///
    /// /!\ The state value is inverted compared to HIGH/LOW electrical value of the pin.
    /// - Inverted button pressed => pin state LOW
    /// - Inverted button released => pin state HIGH
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the Button pin does not exist for this board.
    /// * `IncompatiblePin`: this function will bail an error if the Button pin does not support INPUT mode.
    pub fn new_inverted<T: Into<PinIdOrName>>(board: &dyn Hardware, pin: T) -> Result<Self, Error> {
        Self {
            pin: 0,
            state: Arc::new(RwLock::new(false)),
            invert: true,
            pullup: false,
            protocol: board.get_protocol(),
            handler: Arc::new(RwLock::new(None)),
            events: Default::default(),
        }
        .start_with(board, pin)
    }

    /// Creates an instance of a PULL-UP button attached to a given board:
    /// <https://docs.arduino.cc/tutorials/generic/digital-input-pullup/>
    ///
    /// - Pullup button pressed => pin state LOW
    /// - Pullup button released => pin state HIGH
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the Button pin does not exist for this board.
    /// * `IncompatiblePin`: this function will bail an error if the Button pin does not support INPUT mode.
    pub fn new_pullup<T: Into<PinIdOrName>>(board: &dyn Hardware, pin: T) -> Result<Self, Error> {
        Self {
            pin: 0,
            state: Arc::new(RwLock::new(false)),
            invert: false,
            pullup: true,
            protocol: board.get_protocol(),
            handler: Arc::new(RwLock::new(None)),
            events: Default::default(),
        }
        .start_with(board, pin)
    }

    /// Creates an instance of an inverted PULL-UP button attached to a given board:
    /// <https://docs.arduino.cc/tutorials/generic/digital-input-pullup/>
    ///
    /// /!\ The state value is inverted compared to HIGH/LOW electrical value of the pin
    /// (therefore equivalent to a standard pull-down button)
    /// - Inverted pullup button pressed => pin state HIGH
    /// - Inverted pullup button released => pin state LOW
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the Button pin does not exist for this board.
    /// * `IncompatiblePin`: this function will bail an error if the Button pin does not support INPUT mode.
    pub fn new_inverted_pullup<T: Into<PinIdOrName>>(
        board: &dyn Hardware,
        pin: T,
    ) -> Result<Self, Error> {
        Self {
            pin: 0,
            state: Arc::new(RwLock::new(false)),
            invert: true,
            pullup: true,
            protocol: board.get_protocol(),
            handler: Arc::new(RwLock::new(None)),
            events: Default::default(),
        }
        .start_with(board, pin)
    }

    /// Private helper method shared by constructors.
    fn start_with<T: Into<PinIdOrName>>(
        mut self,
        board: &dyn Hardware,
        pin: T,
    ) -> Result<Self, Error> {
        let pin = board.get_io().read().get_pin(pin)?.clone();

        // Set pin ID and state from pin.
        self.pin = pin.id;
        *self.state.write() = pin.value != 0;

        // Set pin mode to INPUT/PULLUP.
        match self.pullup {
            true => {
                self.protocol.set_pin_mode(self.pin, PinModeId::PULLUP)?;
                self.protocol.get_io().write().get_pin_mut(self.pin)?.value = 1;
            }
            false => {
                self.protocol.set_pin_mode(self.pin, PinModeId::INPUT)?;
            }
        };

        // Set reporting for this pin.
        self.protocol.report_digital(self.pin, true)?;

        // Create a task to listen hardware value and emit events accordingly.
        self.attach();

        Ok(self)
    }

    // ########################################

    /// Returns the pin (id) used by the device.
    pub fn get_pin(&self) -> u8 {
        self.pin
    }

    /// Returns  if the button is configured in PULL-UP mode.
    pub fn is_pullup(&self) -> bool {
        self.pullup
    }

    /// Returns  if the logical button value is inverted.
    pub fn is_inverted(&self) -> bool {
        self.invert
    }

    // ########################################
    // Event related functions

    /// Manually attaches the button with the value change events.
    /// This should never be needed unless you manually `detach()` the button first for some reason
    /// and want it to start being reactive to events again.
    pub fn attach(&self) {
        if self.handler.read().is_none() {
            let self_clone = self.clone();
            *self.handler.write() = Some(
                task::run(async move {
                    loop {
                        let pin_value = self_clone
                            .protocol
                            .get_io()
                            .read()
                            .get_pin(self_clone.pin)?
                            .value
                            != 0;
                        let state_value = *self_clone.state.read();
                        if pin_value != state_value {
                            *self_clone.state.write() = pin_value;

                            // Depending on logical inversion mode, pin_value is inverted.
                            match self_clone.invert {
                                false => self_clone.events.emit(InputEvent::OnChange, pin_value),
                                true => self_clone.events.emit(InputEvent::OnChange, !pin_value),
                            };

                            match self_clone.pullup {
                                true => match pin_value {
                                    true => self_clone.events.emit(InputEvent::OnRelease, ()),
                                    false => self_clone.events.emit(InputEvent::OnPress, ()),
                                },
                                false => match pin_value {
                                    true => self_clone.events.emit(InputEvent::OnPress, ()),
                                    false => self_clone.events.emit(InputEvent::OnRelease, ()),
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
        }
    }

    /// Detaches the interval associated with the button.
    /// This means the button won't react anymore to value changes.
    pub fn detach(&self) {
        if let Some(handler) = self.handler.read().as_ref() {
            handler.abort();
        }
        *self.handler.write() = None
    }

    /// Registers a callback to be executed on a given event on the Button.
    ///
    /// Available events for a button are:
    /// - **`InputEvent::OnChange` | `change`:** Triggered when the button value changes.    
    ///   _The callback must receive the following parameter: `|value: u16| { ... }`_
    /// - **`InputEvent::OnRelease` | `released`:** Triggered when the button value changes.     
    ///   _The callback must receive the void parameter: `|_:()| { ... }`_
    /// - **`InputEvent::OnPress` | `pressed`:** Triggered when the button value changes.     
    ///   _The callback must receive the void parameter: `|_:()| { ... }`_
    ///
    /// # Example
    ///
    /// ```
    /// use hermes_five::devices::{Button, InputEvent};
    /// use hermes_five::hardware::{Board, BoardEvent};
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///     board.on(BoardEvent::OnReady, |board: Board| async move {
    ///
    ///         // Register a Button on pin 2.
    ///         let button = Button::new(&board, 2)?;
    ///         // Triggered function when the button is pressed.
    ///         button.on(InputEvent::OnPress, |_: ()| async move {
    ///             println!("Push button pressed");
    ///             Ok(())
    ///         });
    ///
    ///         // The above code will run forever.
    ///         // <do something useful>
    ///
    ///         // The above code will run forever runs a listener on the pin state under-the-hood.
    ///         // It means the program will run forever listening to the InputEvent,
    ///         // until we detach the device and close the board.
    ///         button.detach();
    ///         board.close();
    ///
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
impl Input for Button {
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

    use crate::hardware::Board;
    use crate::mocks::plugin_io::MockIoProtocol;

    use super::*;

    #[hermes_macros::test]
    fn test_new_button_creation() {
        let board = Board::new(MockIoProtocol::default());
        let button = Button::new(&board, 4);

        assert!(button.is_ok());
        let button = button.unwrap();
        assert_eq!(button.get_pin(), 4);
        assert_eq!(button.get_state().as_bool(), true);
        assert!(!button.is_inverted());
        assert!(!button.is_pullup());

        button.detach();
        board.close();
    }

    #[hermes_macros::test]
    fn test_new_inverted_button_creation() {
        let board = Board::new(MockIoProtocol::default());
        let button = Button::new_inverted(&board, 4);

        assert!(button.is_ok());
        let button = button.unwrap();
        assert_eq!(button.get_pin(), 4);
        assert_eq!(button.get_state().as_bool(), false);
        assert!(button.is_inverted());
        assert!(!button.is_pullup());

        button.detach();
        board.close();
    }

    #[hermes_macros::test]
    fn test_new_pullup_button_creation() {
        let board = Board::new(MockIoProtocol::default());
        let button = Button::new_pullup(&board, 4);

        assert!(button.is_ok());
        let button = button.unwrap();
        assert_eq!(button.get_pin(), 4);
        assert_eq!(button.get_state().as_bool(), true);
        assert!(!button.is_inverted());
        assert!(button.is_pullup());

        button.detach();
        board.close();
    }

    #[hermes_macros::test]
    fn test_new_inverted_pullup_button_creation() {
        let board = Board::new(MockIoProtocol::default());
        let button = Button::new_inverted_pullup(&board, 4);

        assert!(button.is_ok());
        let button = button.unwrap();
        assert_eq!(button.get_pin(), 4);
        assert_eq!(button.get_state().as_bool(), false);
        assert!(button.is_inverted());
        assert!(button.is_pullup());

        button.detach();
        board.close();
    }

    #[hermes_macros::test]
    fn test_button_helper() {
        let board = Board::new(MockIoProtocol::default());
        let button = Button::start_with(
            Button {
                pin: 0,
                state: Arc::new(RwLock::new(false)),
                invert: true,
                pullup: false,
                protocol: board.get_protocol(),
                handler: Arc::new(RwLock::new(None)),
                events: Default::default(),
            },
            &board,
            13,
        );
        assert!(button.is_ok());
        let button = button.unwrap();
        assert_eq!(button.get_pin(), 13);
        assert!(button.handler.read().is_some());
        button.detach();
        assert!(button.handler.read().is_none());
        board.close();
    }

    #[hermes_macros::test]
    fn test_button_inverted_state_logic() {
        let board = Board::new(MockIoProtocol::default());
        let button = Button::new_inverted(&board, 5).unwrap();
        assert_eq!(button.get_state().as_bool(), true);

        button.state.write().clone_from(&true); // Simulate a pressed button
        assert_eq!(button.get_state().as_bool(), false);

        button.detach();
        board.close();
    }

    #[hermes_macros::test]
    fn test_button_events() {
        let board = Board::new(MockIoProtocol::default());
        let button = Button::new(&board, 5).unwrap();

        // CHANGE
        let change_flag = Arc::new(AtomicBool::new(false));
        let moved_change_flag = change_flag.clone();
        button.on(InputEvent::OnChange, move |new_state: bool| {
            let captured_flag = moved_change_flag.clone();
            async move {
                captured_flag.store(new_state, Ordering::SeqCst);
                Ok(())
            }
        });

        // PRESSED
        let pressed_flag = Arc::new(AtomicBool::new(false));
        let moved_pressed_flag = pressed_flag.clone();
        button.on(InputEvent::OnPress, move |_: ()| {
            let captured_flag = moved_pressed_flag.clone();
            async move {
                captured_flag.store(true, Ordering::SeqCst);
                Ok(())
            }
        });

        // RELEASED
        let released_flag = Arc::new(AtomicBool::new(false));
        let moved_released_flag = released_flag.clone();
        button.on(InputEvent::OnRelease, move |_: ()| {
            let captured_flag = moved_released_flag.clone();
            async move {
                captured_flag.store(true, Ordering::SeqCst);
                Ok(())
            }
        });

        assert!(!change_flag.load(Ordering::SeqCst));
        assert!(!pressed_flag.load(Ordering::SeqCst));
        assert!(!released_flag.load(Ordering::SeqCst));

        // Simulate pin state change in the protocol => take value 0xFF
        button
            .protocol
            .get_io()
            .write()
            .get_pin_mut(5)
            .unwrap()
            .value = 0xFF;

        pause!(500);

        assert!(change_flag.load(Ordering::SeqCst));
        assert!(pressed_flag.load(Ordering::SeqCst));
        assert!(!released_flag.load(Ordering::SeqCst));

        // Simulate pin state change in the protocol => takes value 0
        button
            .protocol
            .get_io()
            .write()
            .get_pin_mut(5)
            .unwrap()
            .value = 0;

        pause!(500);

        assert!(!change_flag.load(Ordering::SeqCst)); // change switched back to 0
        assert!(released_flag.load(Ordering::SeqCst));

        button.detach();
        board.close();
    }

    #[hermes_macros::test]
    fn test_inverted_button_events() {
        let board = Board::new(MockIoProtocol::default());
        let button = Button::new_inverted(&board, 5).unwrap();

        // CHANGE
        let change_flag = Arc::new(AtomicBool::new(true));
        let moved_change_flag = change_flag.clone();
        button.on(InputEvent::OnChange, move |new_state: bool| {
            let captured_flag = moved_change_flag.clone();
            async move {
                captured_flag.store(new_state, Ordering::SeqCst);
                Ok(())
            }
        });

        // PRESSED
        let pressed_flag = Arc::new(AtomicBool::new(false));
        let moved_pressed_flag = pressed_flag.clone();
        button.on(InputEvent::OnPress, move |_: ()| {
            let captured_flag = moved_pressed_flag.clone();
            async move {
                captured_flag.store(true, Ordering::SeqCst);
                Ok(())
            }
        });

        // RELEASED
        let released_flag = Arc::new(AtomicBool::new(false));
        let moved_released_flag = released_flag.clone();
        button.on(InputEvent::OnRelease, move |_: ()| {
            let captured_flag = moved_released_flag.clone();
            async move {
                captured_flag.store(true, Ordering::SeqCst);
                Ok(())
            }
        });

        assert!(change_flag.load(Ordering::SeqCst)); // true by default
        assert!(!pressed_flag.load(Ordering::SeqCst));
        assert!(!released_flag.load(Ordering::SeqCst));

        // Simulate pin state change in the protocol => take value 0xFF
        button
            .protocol
            .get_io()
            .write()
            .get_pin_mut(5)
            .unwrap()
            .value = 0xFF;

        pause!(500);

        assert!(!change_flag.load(Ordering::SeqCst)); // changed to false
        assert!(pressed_flag.load(Ordering::SeqCst));
        assert!(!released_flag.load(Ordering::SeqCst));

        // Simulate pin state change in the protocol => takes value 0
        button
            .protocol
            .get_io()
            .write()
            .get_pin_mut(5)
            .unwrap()
            .value = 0;

        pause!(500);

        assert!(change_flag.load(Ordering::SeqCst)); // change switched back to true
        assert!(released_flag.load(Ordering::SeqCst));

        button.detach();
        board.close();
    }

    // #[hermes_macros::test]
    // fn test_pullup_button_events() {
    //     let board = Board::new(MockIoProtocol::default());
    //     let button = Button::new_pullup(&board, 5).unwrap();
    //
    //     // CHANGE
    //     let change_flag = Arc::new(AtomicBool::new(true));
    //     let moved_change_flag = change_flag.clone();
    //     button.on(InputEvent::OnChange, move |new_state: bool| {
    //         let captured_flag = moved_change_flag.clone();
    //         async move {
    //             captured_flag.store(new_state, Ordering::SeqCst);
    //             Ok(())
    //         }
    //     });
    //
    //     // PRESSED
    //     let pressed_flag = Arc::new(AtomicBool::new(false));
    //     let moved_pressed_flag = pressed_flag.clone();
    //     button.on(InputEvent::OnPress, move |_: ()| {
    //         let captured_flag = moved_pressed_flag.clone();
    //         async move {
    //             captured_flag.store(true, Ordering::SeqCst);
    //             Ok(())
    //         }
    //     });
    //
    //     // RELEASED
    //     let released_flag = Arc::new(AtomicBool::new(false));
    //     let moved_released_flag = released_flag.clone();
    //     button.on(InputEvent::OnRelease, move |_: ()| {
    //         let captured_flag = moved_released_flag.clone();
    //         async move {
    //             captured_flag.store(true, Ordering::SeqCst);
    //             Ok(())
    //         }
    //     });
    //
    //     assert!(change_flag.load(Ordering::SeqCst)); // true by default
    //     assert!(!pressed_flag.load(Ordering::SeqCst));
    //     assert!(!released_flag.load(Ordering::SeqCst));
    //
    //     // Simulate pin state change in the protocol => take value 0xFF
    //     button
    //         .protocol
    //         .get_io()
    //         .write()
    //         .get_pin_mut(5)
    //         .unwrap()
    //         .value = 0;
    //
    //     pause!(500);
    //
    //     assert!(!change_flag.load(Ordering::SeqCst)); // changed to false
    //     assert!(pressed_flag.load(Ordering::SeqCst));
    //     assert!(!released_flag.load(Ordering::SeqCst));
    //
    //     // Simulate pin state change in the protocol => takes value 0
    //     button
    //         .protocol
    //         .get_io()
    //         .write()
    //         .get_pin_mut(5)
    //         .unwrap()
    //         .value = 0xFF;
    //
    //     pause!(500);
    //
    //     assert!(change_flag.load(Ordering::SeqCst)); // change switched back to true
    //     assert!(released_flag.load(Ordering::SeqCst));
    //
    //     button.detach();
    //     board.close();
    // }

    #[hermes_macros::test]
    fn test_button_display() {
        let board = Board::new(MockIoProtocol::default());
        let button = Button::new(&board, 4).unwrap();

        assert_eq!(
            format!("{}", button),
            String::from("Button (pin=4) [state=true, pullup=false, inverted=false]")
        );

        button.detach();
        board.close();
    }
}
