use std::time::Duration;

use crate::board::Board;
use crate::devices::{Actuator, Device};
use crate::errors::Error;
use crate::misc::{Easing, Range, State};
use crate::protocols::{Pin, PinModeId, Protocol};
use crate::utils::helpers::MapRange;
use crate::utils::task::TaskHandler;

#[derive(Default, Clone, Copy)]
pub enum ServoType {
    #[default]
    Standard,
    Continuous,
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
    position: u16,
    /// Inner handler to the task running the animation.
    interval: Option<TaskHandler>,
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
            position: 90,
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
        let target = to.clamp(self.range.min, self.range.max);

        // No need to move if last move was already that one.
        if self.last_move.is_some() && self.last_move.unwrap().position == target {
            return Ok(self);
        }

        // Stops any animation running.
        self.stop();

        self.update(State::from(target))
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

    // @todo move this to device
    pub fn pin(&self) -> Result<Pin, Error> {
        let lock = self.protocol.hardware().read();
        Ok(lock.get_pin(self.pin)?.clone())
    }
}

// @todo make this a derive macro
impl Device for Servo {}

impl Actuator for Servo {
    /// Update the Servo position.
    fn update(&mut self, target: State) -> Result<&Self, Error> {
        let target: u16 = target.as_integer() as u16;

        // Map value from degree_range to pwm_range scale.
        let microseconds = target.map(
            self.degree_range.min,
            self.degree_range.max,
            self.pwm_range.min,
            self.pwm_range.max,
        );

        self.protocol.analog_write(self.pin, microseconds)?;
        self.position = target;
        self.last_move = Some(Move {
            position: self.position,
        });

        Ok(self)
    }

    fn animate(
        &mut self,
        target: State,
        duration: Duration,
        easing: Easing,
    ) -> Result<&Self, Error> {
        todo!()
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
            position: self.position,
            last_move: self.last_move,
            is_moving: false,
            interval: None,
        }
    }
}
