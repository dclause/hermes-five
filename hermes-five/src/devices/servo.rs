use async_trait::async_trait;

use crate::board::Board;
use crate::devices::{Actuator, Device};
use crate::errors::Error;
use crate::errors::HardwareError::IncompatibleMode;
use crate::pause_sync;
use crate::protocols::{Pin, PinModeId, Protocol};
use crate::utils::events::{EventHandler, EventManager};
use crate::utils::Range;
use crate::utils::scale::Scalable;
use crate::utils::task::TaskHandler;

#[derive(Default, Clone, Copy, Debug)]
pub enum ServoType {
    #[default]
    Standard,
    // Continuous,
}

#[derive(Debug)]
pub struct Servo {
    protocol: Box<dyn Protocol>,
    pin: u16,

    /// The servo range limitation in the physical world (default: [0, 180]).
    range: Range<u16>,
    /// The servo PWN range for control  (default: [544, 2400]).
    pwm_range: Range<u16>,
    /// The servo theoretical degree of movement  (default: [0, 180]).
    degree_range: Range<u16>,
    /// The servo type (default: ServoType::Standard).
    servo_type: ServoType,
    /// The servo default position to it initialize (default: 90)
    default: u16,
    /// Indicates of the servo is currently moving.
    is_moving: bool, // @todo remove?

    // Because we are an actuator:
    /// Current position
    state: u16,
    /// Last move done by the servo.
    previous: u16,
    /// Inner handler to the task running the animation.
    interval: Option<TaskHandler>,

    // Because we are an emitter:
    events: EventManager,
}

impl Servo {
    pub fn new(board: &Board, pin: u16, default: u16) -> Result<Self, Error> {
        let pwm_range = Range::from([600, 2400]);

        let mut servo = Self {
            protocol: board.protocol(),
            pin,
            range: Range::from([0, 180]),
            pwm_range,
            degree_range: Range::from([0, 180]),
            servo_type: ServoType::default(),
            default,
            is_moving: false,
            previous: default,
            interval: None,
            events: EventManager::default(),
            state: default,
        };

        // --
        // The following may seem tedious, but it ensures we attach the servo with the default value
        // already set.
        // Check if SERVO MODE exists for this pin.
        servo
            .pin()?
            .supports_mode(PinModeId::SERVO)
            .ok_or(IncompatibleMode {
                pin,
                mode: PinModeId::SERVO,
                context: "create a new Servo device",
            })?;
        servo.protocol.servo_config(pin, pwm_range)?;
        servo.to(servo.default)?;
        servo.protocol.set_pin_mode(pin, PinModeId::SERVO)?;
        pause_sync!(100);
        Ok(servo)
    }

    /// Set the Servo range limitation in degree.
    /// This guarantee the servo to stays in the given range at any time.
    ///
    /// - No matter the order given, the range will always have min <= max
    /// - No matter the values given, the range will always stay within the Servo `degree_range`.
    ///
    /// # Parameters
    /// * `range`: the requested range
    pub fn with_range<R: Into<Range<u16>>>(mut self, range: R) -> Self {
        let input = range.into();

        // Rearrange value: min <= max.
        let input = Range {
            start: input.start.min(input.end),
            end: input.end.max(input.start),
        };

        // Clamp the range into the degree_range.
        self.range = Range {
            start: input
                .start
                .clamp(self.degree_range.start, self.degree_range.end),
            end: input
                .end
                .clamp(self.degree_range.start, self.degree_range.end),
        };
        // Clamp the default position inside the range.
        self.default = self.default.clamp(self.range.start, self.range.end);

        self
    }

    /// Set the theoretical range of degrees of movement for the servo (some servos can range from 0 to 90°, 180°, 270°, 360°, etc...).
    ///
    /// - No matter the order given, the range will always have min <= max
    /// - This may impact the `range` since it will always stay within the given `degree_range`.
    ///
    /// # Parameters
    /// * `degree_range`: the requested range
    pub fn with_degree_range<R: Into<Range<u16>>>(mut self, degree_range: R) -> Self {
        let input = degree_range.into();

        // Rearrange value: min <= max.
        let input = Range {
            start: input.start.min(input.end),
            end: input.end.max(input.start),
        };

        self.degree_range = input;

        // Clamp the range into the degree_range.
        self.range = Range {
            start: self
                .range
                .start
                .clamp(self.degree_range.start, self.degree_range.end),
            end: self
                .range
                .end
                .clamp(self.degree_range.start, self.degree_range.end),
        };
        // Clamp the default position inside the range.
        self.default = self.default.clamp(self.range.start, self.range.end);

        self
    }

    pub fn set_pwn_range<R: Into<Range<u16>>>(mut self, pwm_range: R) -> Result<Self, Error> {
        let input = pwm_range.into();
        self.pwm_range = input;
        self.protocol.servo_config(self.pin, input)?;
        Ok(self)
    }

    /// Move the servo to the requested position at max speed.
    pub fn to(&mut self, to: u16) -> Result<&Self, Error> {
        // Clamp the request within the Servo range.
        let state: u16 = to.clamp(self.range.start, self.range.end);

        // Stops any animation running.
        self.stop();

        self.set_state(state)?;
        Ok(self)
    }

    /// Stops the servo.
    /// Any animation running will be stopped after the current running step is executed.
    /// Any simple move running will be stopped at end position.
    pub fn stop(&self) {
        match &self.interval {
            None => {}
            Some(handler) => handler.abort(),
        }
    }

    pub fn sweep() {
        // // Swipe the servo.
        // loop {
        //     servo.to(0).unwrap();
        //     pause!(1000);
        //     servo.to(180).unwrap();
        //     pause!(1000);
        // }
    }

    // @todo move this to device
    pub fn pin(&self) -> Result<Pin, Error> {
        let lock = self.protocol.get_hardware().read();
        Ok(lock.get_pin(self.pin)?.clone())
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
    pub async fn on<S, F, T, Fut>(&self, event: S, callback: F) -> EventHandler
    where
        S: Into<String>,
        T: 'static + Send + Sync + Clone,
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), Error>> + Send + 'static,
    {
        self.events.on(event, callback)
    }
}

// @todo make this a derive macro
impl Device for Servo {}

#[async_trait]
impl Actuator for Servo {
    /// Update the Servo position.
    fn set_state(&mut self, state: u16) -> Result<(), Error> {
        self.state = state;

        // No need to move if last move was already that one.
        if self.previous == self.state {
            // return Ok(());
        }

        let state: f64 = state.scale(
            self.degree_range.start,
            self.degree_range.end,
            self.pwm_range.start,
            self.pwm_range.end,
        );
        self.protocol.analog_write(self.pin, state as u16)?;
        self.previous = self.state;

        Ok(())
    }

    /// Internal only (used by [`Animation`]).
    fn get_state(&self) -> u16 {
        self.state
    }
}

impl Drop for Servo {
    fn drop(&mut self) {
        let _ = self.to(self.default);
    }
}

impl Clone for Servo {
    fn clone(&self) -> Self {
        Self {
            protocol: self.protocol.clone(),
            pin: self.pin,
            range: self.range,
            pwm_range: self.pwm_range,
            degree_range: self.degree_range,
            servo_type: self.servo_type,
            default: self.default,
            state: self.state,
            previous: self.previous,
            is_moving: self.is_moving,
            // do not clone
            interval: None,
            events: EventManager::default(),
        }
    }
}
