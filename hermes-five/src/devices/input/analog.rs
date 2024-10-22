use std::fmt::{Display, Formatter};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::devices::input::{Input, InputEvent};
use crate::devices::Device;
use crate::errors::Error;
use crate::protocols::{PinIdOrName, PinModeId, Protocol};
use crate::utils::events::{EventHandler, EventManager};
use crate::utils::task::TaskHandler;
use crate::utils::{task, State};
use crate::{pause, Board};

/// Represents an analog sensor of unspecified type: an [`Input`] [`Device`] that reads analog values
/// from an ANALOG compatible pin.
/// https://docs.arduino.cc/built-in-examples/analog/AnalogInput
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct AnalogInput {
    // ########################################
    // # Basics
    /// The pin (id) of the [`Board`] used to read the analog value.
    pin: u16,
    /// The current AnalogInput state.
    #[cfg_attr(feature = "serde", serde(with = "crate::devices::arc_rwlock_serde"))]
    state: Arc<RwLock<u16>>,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol: Box<dyn Protocol>,
    /// Inner handler to the task running the button value check.
    #[cfg_attr(feature = "serde", serde(skip))]
    handler: Arc<RwLock<Option<TaskHandler>>>,
    /// The event manager for the AnalogInput.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager,
}

impl AnalogInput {
    /// Creates an instance of an [`AnalogInput`] attached to a given board:
    /// https://docs.arduino.cc/built-in-examples/analog/AnalogInput/
    ///
    /// # Parameters
    /// * `board`: the [`Board`] which the AnalogInput is attached to
    /// * `analog_pin`: the analog pin used to read the AnalogInput value
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the AnalogInput pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the AnalogInput pin does not support ANALOG mode.
    pub fn new<T: Into<PinIdOrName>>(board: &Board, analog_pin: T) -> Result<Self, Error> {
        let pin = board.get_hardware().get_pin(analog_pin.into())?.clone();

        let mut sensor = Self {
            pin: pin.id,
            state: Arc::new(RwLock::new(pin.value)),
            protocol: board.get_protocol(),
            handler: Arc::new(RwLock::new(None)),
            events: Default::default(),
        };

        // Set pin mode to ANALOG.
        sensor
            .protocol
            .set_pin_mode(sensor.pin, PinModeId::ANALOG)?;

        // Attaches the event handler.
        board.attach();
        sensor.attach();

        Ok(sensor)
    }

    // ########################################
    // Event related functions

    /// Manually attaches the AnalogInput with the value change events.
    /// This should never be needed unless you manually `detach()` the AnalogInput first for some reason
    /// and want it to start being reactive to events again.
    pub fn attach(&self) {
        if self.handler.read().is_none() {
            let self_clone = self.clone();
            *self.handler.write() = Some(
                task::run(async move {
                    loop {
                        let pin_value = self_clone
                            .protocol
                            .get_hardware()
                            .read()
                            .get_pin(self_clone.pin)?
                            .value;
                        let state_value = self_clone.state.read().clone();
                        if pin_value != state_value {
                            *self_clone.state.write() = pin_value;
                            self_clone.events.emit(InputEvent::OnChange, pin_value);
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

    /// Detaches the interval associated with the AnalogInput.
    /// This means the AnalogInput won't react anymore to value changes.
    pub fn detach(&self) {
        if let Some(handler) = self.handler.read().as_ref() {
            handler.abort();
        }
    }

    /// Registers a callback to be executed on a given event on the AnalogInput.
    ///
    /// Available events for an AnalogInput are:
    /// * `change`: Triggered when the AnalogInput value changes. To use it, register though the [`Self::on()`] method.
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

impl Display for AnalogInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AnalogInput (pin={}) [state={}]",
            self.pin,
            self.state.read(),
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for AnalogInput {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Input for AnalogInput {
    fn get_state(&self) -> State {
        State::from(*self.state.read())
    }
}

#[cfg(test)]
mod tests {
    use crate::devices::input::analog::AnalogInput;
    use crate::devices::input::Input;
    use crate::devices::input::InputEvent;
    use crate::mocks::protocol::MockProtocol;
    use crate::{pause, Board};
    use std::sync::atomic::{AtomicU16, Ordering};
    use std::sync::Arc;

    #[hermes_macros::test]
    fn test_new_analog_input() {
        let board = Board::from(MockProtocol::default());
        let sensor = AnalogInput::new(&board, 14).unwrap();
        assert_eq!(sensor.pin, 14);
        assert_eq!(sensor.get_state().as_integer(), 100);
        sensor.detach();

        let sensor = AnalogInput::new(&board, "A22").unwrap();
        assert_eq!(sensor.pin, 22);
        assert_eq!(sensor.get_state().as_integer(), 222);

        board.detach();
        sensor.detach();
    }

    #[hermes_macros::test]
    fn test_analog_display() {
        let board = Board::from(MockProtocol::default());
        let sensor = AnalogInput::new(&board, "A15").unwrap();
        assert_eq!(sensor.get_state().as_integer(), 200);
        assert_eq!(
            format!("{}", sensor),
            String::from("AnalogInput (pin=15) [state=200]")
        );

        board.detach();
        sensor.detach();
    }

    #[hermes_macros::test]
    fn test_analog_events() {
        let pin = "A14";
        let board = Board::from(MockProtocol::default());
        let sensor = AnalogInput::new(&board, pin).unwrap();
        assert_eq!(sensor.get_state().as_integer(), 100);

        // CHANGE
        let change_flag = Arc::new(AtomicU16::new(100));
        let moved_change_flag = change_flag.clone();
        sensor.on(InputEvent::OnChange, move |new_state: u16| {
            let captured_flag = moved_change_flag.clone();
            async move {
                captured_flag.store(new_state, Ordering::SeqCst);
                Ok(())
            }
        });

        assert_eq!(change_flag.load(Ordering::SeqCst), 100);

        // Simulate pin state change in the protocol => take value 0xFF
        sensor
            .protocol
            .get_hardware()
            .write()
            .get_pin_mut(pin)
            .unwrap()
            .value = 0xFF;

        pause!(500);
        assert_eq!(change_flag.load(Ordering::SeqCst), 0xFF);

        board.detach();
        sensor.detach();
    }
}
