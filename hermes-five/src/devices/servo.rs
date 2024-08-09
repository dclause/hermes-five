use std::sync::Arc;

use async_trait::async_trait;

use crate::board::Board;
use crate::devices::{Actuator, Device};
use crate::errors::Error;
use crate::errors::HardwareError::IncompatibleMode;
use crate::pause_sync;
use crate::protocols::{Pin, PinModeId, Protocol};
use crate::utils::Range;
use crate::utils::scale::Scalable;
use crate::utils::task::TaskHandler;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Clone, Copy, Debug)]
pub enum ServoType {
    #[default]
    Standard,
    // Continuous,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Servo {
    // ########################################
    // # Basics
    /// The pin (id) of the board [`Board`] used to control the Servo.
    pin: u16,
    /// The current Servo state.
    state: u16,
    /// The LED default value (default: ON).
    default: u16,

    // ########################################
    // # Settings
    /// The servo type (default: ServoType::Standard).
    servo_type: ServoType,
    /// The servo range limitation in the physical world (default: [0, 180]).
    range: Range<u16>,
    /// The servo PWN range for control  (default: [544, 2400]).
    pwm_range: Range<u16>,
    /// The servo theoretical degree of movement  (default: [0, 180]).
    degree_range: Range<u16>,

    // ########################################
    // # Volatile utility data.
    /// Last move done by the servo.
    #[cfg_attr(feature = "serde", serde(skip))]
    previous: u16,
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol: Box<dyn Protocol>,
    /// Inner handler to the task running the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    interval: Arc<Option<TaskHandler>>,
}

impl Servo {
    pub fn new(board: &Board, pin: u16, default: u16) -> Result<Self, Error> {
        let pwm_range = Range::from([600, 2400]);

        let mut servo = Self {
            pin,
            state: default,
            default,
            servo_type: ServoType::default(),
            range: Range::from([0, 180]),
            pwm_range,
            degree_range: Range::from([0, 180]),
            previous: default,
            protocol: board.get_protocol(),
            interval: Arc::new(None),
        };

        // --
        // The following may seem tedious, but it ensures we attach the servo with the default value
        // already set.
        // Check if SERVO MODE exists for this pin.
        servo
            .get_pin_info()?
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

    /// Move the servo to the requested position at max speed.
    pub fn to(&mut self, to: u16) -> Result<&Self, Error> {
        // Clamp the request within the Servo range.
        let state: u16 = to.clamp(self.range.start, self.range.end);

        // Stops any animation running.
        self.stop();

        self._set_state(state)?;
        Ok(self)
    }

    /// Stops the servo.
    /// Any animation running will be stopped after the current running step is executed.
    /// Any simple move running will be stopped at end position.
    pub fn stop(&self) {
        match &self.interval.as_ref() {
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

    // ########################################
    // Setters and Getters.

    /// Retrieves the PIN (id) used to control the LED.
    pub fn get_pin(&self) -> u16 {
        self.pin
    }

    /// Retrieves [`Pin`] information.
    pub fn get_pin_info(&self) -> Result<Pin, Error> {
        let lock = self.protocol.get_hardware().read();
        Ok(lock.get_pin(self.pin)?.clone())
    }

    /// Retrieves the servo type.
    pub fn get_type(&self) -> ServoType {
        self.servo_type
    }

    /// Retrieves the servo motion range limitation in degree.
    ///
    /// A servo has a physical range (cf [`Servo::degree_range`]) corresponding to a command range
    /// limitation (cf [`Servo::pwn_range`]). Those are intrinsic top the servo itself. On the contrary,
    /// the motion range limitation here is a limitation you want to set for your servo because of how
    /// it is used in your robot: for example an arm that can turn only 20-40° in motion range.
    pub fn get_range(&self) -> Range<u16> {
        self.range
    }

    /// Set the Servo motion range limitation in degree. This guarantee the servo to stays in the given
    /// range at any time.
    ///
    /// - No matter the order given, the range will always have min <= max
    /// - No matter the values given, the range will always stay within the Servo `degree_range`.
    ///
    /// # Parameters
    /// * `range`: the range limitation
    pub fn set_range<R: Into<Range<u16>>>(mut self, range: R) -> Self {
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
    /// * `degree_range`: the range limitation
    pub fn set_degree_range<R: Into<Range<u16>>>(mut self, degree_range: R) -> Self {
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

    /// Set the theoretical range of pwm controls the servo response to.
    ///
    /// # Parameters
    /// * `pwm_range`: the range limitation
    pub fn set_pwn_range<R: Into<Range<u16>>>(mut self, pwm_range: R) -> Result<Self, Error> {
        let input = pwm_range.into();
        self.pwm_range = input;
        self.protocol.servo_config(self.pin, input)?;
        Ok(self)
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for Servo {
    fn set_board(&mut self, board: &Board) {
        self.protocol = board.get_protocol();
    }
}

#[async_trait]
#[cfg_attr(feature = "serde", typetag::serde)]
impl Actuator for Servo {
    /// Update the Servo position.
    fn _set_state(&mut self, state: u16) -> Result<(), Error> {
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

    /// Retrieves the actuator current state.
    fn get_state(&self) -> u16 {
        self.state
    }

    /// Retrieves the actuator default (or neutral) state.
    fn get_default(&self) -> u16 {
        self.default
    }

    /// Indicates the busy status, ie if the device is running an animation.
    fn is_busy(&self) -> bool {
        self.interval.is_some()
    }
}

// impl Drop for Servo {
//     fn drop(&mut self) {
//         let _ = self._set_state(self.get_default());
//     }
// }
