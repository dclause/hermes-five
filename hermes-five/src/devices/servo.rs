use std::panic::UnwindSafe;
use std::time::SystemTime;

use async_trait::async_trait;

use crate::board::Board;
use crate::devices::{Actuator, Device};
use crate::errors::Error;
use crate::pause;
use crate::protocols::{Pin, PinModeId, Protocol};
use crate::utils::{Easing, Range, task};
use crate::utils::events::{EventHandler, EventManager};
use crate::utils::scale::Scalable;
use crate::utils::task::TaskHandler;

#[derive(Default, Clone, Copy)]
pub enum ServoType {
    #[default]
    Standard,
    // Continuous,
}

#[derive(Clone, Copy)]
struct Move {
    position: u16,
}

pub struct Servo {
    protocol: Box<dyn Protocol>,
    pin: u16,

    /// The servo range limitation in the physical world (default: [0, 180]).
    range: Range,
    /// The servo PWN range for control  (default: [544, 2400]).
    pwm_range: Range,
    /// The servo theoretical degree of movement  (default: [0, 180]).
    degree_range: Range,
    /// The servo type (default: ServoType::Standard).
    servo_type: ServoType,
    /// The servo default position to it initialize (default: 90)
    default: u16,
    /// Last move done by the servo.
    last_move: Option<Move>,
    /// Indicates of the servo is currently moving.
    is_moving: bool, // @todo remove?

    // Because we are an actuator:
    /// Current position
    state: u16,
    /// Inner handler to the task running the animation.
    interval: Option<TaskHandler>,

    // Because we are an emitter:
    /// The event manager for the board.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager,
}

impl Servo {
    pub fn new(board: &Board, pin: u16) -> Result<Self, Error> {
        // Set pin mode to SERVO
        let mut protocol = board.protocol();
        protocol.set_pin_mode(pin, PinModeId::SERVO)?;
        protocol.servo_config(pin, Range::from([544, 2400]))?;

        Ok(Self {
            protocol: board.protocol(),
            pin,
            range: Range::from([0, 180]),
            pwm_range: Range::from([544, 2400]),
            degree_range: Range::from([0, 180]),
            servo_type: ServoType::default(),
            default: 90,
            is_moving: false,
            last_move: None,
            interval: None,
            events: EventManager::default(),
            state: 90,
        })
    }

    /// Set the Servo range limitation in degree.
    /// This guarantee the servo to stays in the given range at any time.
    ///
    /// - No matter the order given, the range will always have min <= max
    /// - No matter the values given, the range will always stay within the Servo `degree_range`.
    ///
    /// # Parameters
    /// * `range`: the requested range
    pub fn with_range<R: Into<Range>>(mut self, range: R) -> Result<Self, Error> {
        let input = range.into();

        // Rearrange value: min <= max.
        let input = Range {
            min: input.min.min(input.max),
            max: input.max.max(input.min),
        };

        // Clamp the range into the degree_range.
        self.range = Range {
            min: input
                .min
                .clamp(self.degree_range.min, self.degree_range.max),
            max: input
                .max
                .clamp(self.degree_range.min, self.degree_range.max),
        };

        Ok(self)
    }

    /// Set the theoretical range of degrees of movement for the servo (some servos can range from 0 to 90°, 180°, 270°, 360°, etc...).
    ///
    /// - No matter the order given, the range will always have min <= max
    /// - This may impact the `range` since it will always stay within the given `degree_range`.
    ///
    /// # Parameters
    /// * `degree_range`: the requested range
    pub fn with_degree_range<R: Into<Range>>(mut self, degree_range: R) -> Result<Self, Error> {
        let input = degree_range.into();

        // Rearrange value: min <= max.
        let input = Range {
            min: input.min.min(input.max),
            max: input.max.max(input.min),
        };

        self.degree_range = input;

        // Clamp the range into the degree_range.
        self.range = Range {
            min: self
                .range
                .min
                .clamp(self.degree_range.min, self.degree_range.max),
            max: self
                .range
                .max
                .clamp(self.degree_range.min, self.degree_range.max),
        };

        self.protocol.servo_config(self.pin, self.degree_range)?;

        Ok(self)
    }

    /// Move the servo to the requested position at max speed.
    pub fn to(&mut self, to: u16) -> Result<&Self, Error> {
        // Clamp the request within the Servo range.
        self.state = to.clamp(self.range.min, self.range.max);

        // No need to move if last move was already that one.
        if self.last_move.is_some() && self.last_move.unwrap().position == self.state {
            return Ok(self);
        }

        // Stops any animation running.
        self.stop();

        self.update()?;
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
        let lock = self.protocol.hardware().read();
        Ok(lock.get_pin(self.pin)?.clone())
    }

    pub fn animate_sync(&mut self, target: u16, duration: u32, easing: Easing) {
        let animation_start_value = self.state;
        let animation_end_value = target;

        let fps = 40f32;
        let tick_ms = (1000f32 / fps) as u32;
        let mut t_ms = 0u32;

        while t_ms < duration {
            // Current time between (0 - 1).
            let normalized_t = (t_ms as f32) / (duration as f32);
            // Current value between (0 - 1)
            self.state = easing.call(normalized_t).scale(
                0f32,
                1f32,
                animation_start_value as f32,
                animation_end_value as f32,
            ) as u16;

            // Update servo position
            self.update().unwrap();

            // Wait for the next tick
            std::thread::sleep(std::time::Duration::from_millis(tick_ms as u64));
            // pause!(tick_ms);

            t_ms += tick_ms;
        }
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

// @todo make this a derive macro
impl Device for Servo {}

#[async_trait]
impl Actuator for Servo {
    /// Update the Servo position.
    fn update(&mut self) -> Result<(), Error> {
        // Map value from degree_range to pwm_range scale.
        let microseconds = self.state.scale(
            self.degree_range.min,
            self.degree_range.max,
            self.pwm_range.min,
            self.pwm_range.max,
        );

        self.protocol.analog_write(self.pin, microseconds)?;
        self.last_move = Some(Move {
            position: self.state,
        });

        Ok(())
    }

    async fn animate(&mut self, target: u16, duration: u32, easing: Easing) {
        let mut self_clone = self.clone();

        self.stop();

        self.interval = Some(
            task::run(async move {
                let start = SystemTime::now();

                let animation_start_value = self_clone.state;
                let animation_end_value = target;

                let fps = 40f32;
                let tick_ms = (1000f32 / fps) as u32;
                let mut t_ms = 0u32;

                while t_ms < duration {
                    // Current time between (0 - 1).
                    let normalized_t = (t_ms as f32) / (duration as f32);
                    // Current value between (0 - 1)
                    self_clone.state = easing.call(normalized_t).scale(
                        0f32,
                        1f32,
                        animation_start_value as f32,
                        animation_end_value as f32,
                    ) as u16;

                    // Update servo position
                    self_clone.update()?;

                    // Wait for the next tick
                    pause!(tick_ms);

                    t_ms += tick_ms;
                }

                let end = SystemTime::now();
                let elapsed = end.duration_since(start).unwrap().as_millis();
                println!("Animate duration: {}", elapsed);

                let callback_clone = self_clone.clone();
                self_clone.events.emit("complete", callback_clone).await;
                Ok(())
            })
            .await
            .unwrap(),
        );
    }
}

impl Clone for Servo {
    fn clone(&self) -> Self {
        Self {
            protocol: self.protocol.clone(),
            pin: self.pin,
            range: self.range,
            pwm_range: self.range,
            degree_range: self.degree_range,
            servo_type: self.servo_type,
            default: self.default,
            state: self.state,
            last_move: self.last_move,
            is_moving: self.is_moving,
            // do not clone
            interval: None,
            events: EventManager::default(),
        }
    }
}
