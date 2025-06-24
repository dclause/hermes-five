use parking_lot::RwLock;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::devices::input::{Input, InputEvent};
use crate::devices::Device;
use crate::errors::Error;
use crate::hardware::Hardware;
use crate::io::{IoProtocol, PinIdOrName, PinModeId};
use crate::pause;
use crate::utils::{task, EventManager, GenericResult, State, TaskHandler};

/// Represents a digital sensor of unspecified type: an [`Input`] [`Device`] that reads digital values
/// from an INPUT compatible pin.
/// <https://docs.arduino.cc/built-in-examples/digital/DigitalInput>
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct DigitalInput {
    // ########################################
    // # Basics
    /// The pin (id) of the [`Board`] used to read the digital value.
    pin: u8,
    /// The current digital state.
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::serde_arc_atomic"))]
    state: Arc<AtomicBool>,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::utils::serde_arc_protocol", skip_serializing)
    )]
    protocol: Arc<dyn IoProtocol>,
    /// Inner handler to the task running the button value check.
    #[cfg_attr(feature = "serde", serde(skip))]
    handler: Arc<RwLock<Option<TaskHandler>>>,
    /// The event manager for the DigitalInput.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager<InputEvent, bool>,
}

impl DigitalInput {
    /// Creates an instance of a [`DigitalInput`] attached to a given board.
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the DigitalInput pin does not exist for this board.
    /// * `IncompatiblePin`: this function will bail an error if the DigitalInput pin does not support ANALOG mode.
    pub fn new<T: Into<PinIdOrName>>(board: &dyn Hardware, pin: T) -> Result<Self, Error> {
        let pin = board.get_io().read().get_pin(pin)?.clone();

        let sensor = Self {
            pin: pin.id,
            state: Arc::new(AtomicBool::new(pin.value != 0)),
            protocol: board.get_protocol(),
            handler: Arc::new(RwLock::new(None)),
            events: EventManager::default(),
        };

        // Set pin mode to INPUT.
        sensor.protocol.set_pin_mode(sensor.pin, PinModeId::INPUT)?;

        // Set reporting for this pin.
        sensor.protocol.report_digital(sensor.pin, true)?;

        // Attaches the event handler.
        sensor.attach();

        Ok(sensor)
    }

    // ########################################
    // Getters and Setters

    /// Returns the pin (id) used by the device.
    pub fn get_pin(&self) -> u8 {
        self.pin
    }

    // ########################################
    // Event related functions

    /// Manually attaches the DigitalInput with the value change events.
    /// This should never be needed unless you manually `detach()` the DigitalInput first for some reason
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
                        let state_value = self_clone.state.load(Ordering::SeqCst);
                        if pin_value != state_value {
                            self_clone.state.store(pin_value, Ordering::SeqCst);
                            self_clone.events.emit(InputEvent::OnChange, pin_value);
                            match pin_value {
                                true => self_clone.events.emit(InputEvent::OnHigh, pin_value),
                                false => self_clone.events.emit(InputEvent::OnLow, pin_value),
                            }
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

    /// Detaches the interval associated with the DigitalInput.
    /// This means the DigitalInput won't react anymore to value changes.
    pub fn detach(&self) {
        if let Some(handler) = self.handler.read().as_ref() {
            handler.abort();
        }
        *self.handler.write() = None
    }

    /// Registers a callback to be executed on a given event on the DigitalInput.
    ///
    /// Available events for a button are:
    /// - **`InputEvent::OnChange` | `change`:** Triggered when the input value changes.    
    ///   _The callback must receive the following parameter: `|value: bool| { ... }`_
    /// - **`InputEvent::OnHigh` | `high`:** Triggered when the input value changes.     
    ///   _The callback must receive the void parameter: `|_:()| { ... }`_
    ///- **`InputEvent::OnLow` | `low`:** Triggered when the input value changes.    
    ///   _The callback must receive the void parameter: `|_:()| { ... }`_
    ///
    /// # Example
    ///
    ///```
    /// use hermes_five::devices::{DigitalInput, InputEvent};
    /// use hermes_five::hardware::{Board, BoardEvent};
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::start().unwrap();
    ///     board.on(BoardEvent::OnReady, |board: Board| async move {
    ///
    ///         // Register a sensor on pin 7.
    ///         let sensor = DigitalInput::new(&board, 7)?;
    ///         // Triggered function when the sensor state changes.
    ///         sensor.on(InputEvent::OnChange, |value: bool| async move {
    ///             println!("Sensor value changed: {}", value);
    ///         });
    ///
    ///         // The above code will run forever.
    ///         // <do something useful>
    ///
    ///         // The above code will run forever runs a listener on the pin state under-the-hood.
    ///         // It means the program will run forever listening to the InputEvent,
    ///         // until we detach the device and close the board.
    ///         sensor.detach();
    ///         board.disconnect().unwrap();
    ///
    ///         Ok(())
    ///     });
    /// }
    /// ```
    pub fn on<F, Fut, R>(&self, event: InputEvent, handler: F)
    where
        F: Fn(bool) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Into<GenericResult>,
    {
        self.events.on(event, handler);
    }
}

impl Display for DigitalInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DigitalInput (pin={}) [state={}]",
            self.pin,
            self.state.load(Ordering::SeqCst),
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for DigitalInput {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Input for DigitalInput {
    fn get_state(&self) -> State {
        State::from(self.state.load(Ordering::SeqCst))
    }
}

#[cfg(test)]
mod tests {
    use crate::devices::input::digital::DigitalInput;
    use crate::devices::input::Input;
    use crate::devices::input::InputEvent;
    use crate::hardware::Board;
    use crate::mocks::MockProtocol;
    use crate::pause;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[hermes_five_macros::test]
    fn test_new_digital_input() {
        let board = Board::new(MockProtocol::default());
        let sensor = DigitalInput::new(&board, 2).unwrap();
        assert_eq!(sensor.get_pin(), 2);
        assert!(sensor.get_state().as_bool());
        sensor.detach();

        let sensor = DigitalInput::new(&board, "D3").unwrap();
        assert_eq!(sensor.get_pin(), 3);
        assert!(sensor.get_state().as_bool());

        sensor.detach();
        board.disconnect().unwrap();
    }

    #[hermes_five_macros::test]
    fn test_digital_display() {
        let board = Board::new(MockProtocol::default());
        let sensor = DigitalInput::new(&board, "D5").unwrap();
        assert!(!sensor.get_state().as_bool());
        assert_eq!(
            format!("{}", sensor),
            String::from("DigitalInput (pin=5) [state=false]")
        );

        sensor.detach();
        board.disconnect().unwrap();
    }

    #[hermes_five_macros::test]
    fn test_digital_events() {
        let board = Board::new(MockProtocol::default());
        let button = DigitalInput::new(&board, 5).unwrap();

        // CHANGE
        let change_flag = Arc::new(AtomicBool::new(false));
        let moved_change_flag = change_flag.clone();
        button.on(InputEvent::OnChange, move |new_state: bool| {
            let captured_flag = moved_change_flag.clone();
            async move {
                captured_flag.store(new_state, Ordering::SeqCst);
            }
        });

        // HIGH
        let high_flag = Arc::new(AtomicBool::new(false));
        let moved_high_flag = high_flag.clone();
        button.on(InputEvent::OnHigh, move |_: bool| {
            let captured_flag = moved_high_flag.clone();
            async move {
                captured_flag.store(true, Ordering::SeqCst);
            }
        });

        // LOW
        let low_flag = Arc::new(AtomicBool::new(false));
        let moved_low_flag = low_flag.clone();
        button.on(InputEvent::OnLow, move |_: bool| {
            let captured_flag = moved_low_flag.clone();
            async move {
                captured_flag.store(true, Ordering::SeqCst);
            }
        });

        assert!(!change_flag.load(Ordering::SeqCst));
        assert!(!high_flag.load(Ordering::SeqCst));
        assert!(!low_flag.load(Ordering::SeqCst));

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
        assert!(high_flag.load(Ordering::SeqCst));
        assert!(!low_flag.load(Ordering::SeqCst));

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
        assert!(low_flag.load(Ordering::SeqCst));

        button.detach();
        board.disconnect().unwrap();
    }
}
