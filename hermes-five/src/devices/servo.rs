use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::SystemTime;

use log::debug;
use parking_lot::RwLock;

use crate::{pause, pause_sync};
use crate::animation::{Animation, Keyframe, Track};
use crate::board::Board;
use crate::devices::{Actuator, Device};
use crate::errors::{Error, StateError};
use crate::errors::HardwareError::IncompatibleMode;
use crate::protocols::{Pin, PinModeId, Protocol};
use crate::utils::{Easing, Range, State, task};
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
    #[cfg_attr(feature = "serde", serde(with = "crate::devices::arc_rwlock_serde"))]
    state: Arc<RwLock<u16>>,
    /// The LED default value (default: ON).
    default: u16,

    // ########################################
    // # Settings
    /// The servo type (default: ServoType::Standard).
    servo_type: ServoType,
    /// The servo range limitation in the physical world (default: [0, 180]).
    range: Range<u16>,
    /// The servo PWN range for control  (default: [600, 2400]).
    pwm_range: Range<u16>,
    /// The servo theoretical degree of movement  (default: [0, 180]).
    degree_range: Range<u16>,
    /// Specifies if the servo command is inverted (default: false).
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "crate::utils::is_default")
    )]
    #[cfg_attr(feature = "serde", serde(default))]
    inverted: bool,
    /// Specifies if the servo should auto detach itself after a given delay (default: false).
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "crate::utils::is_default")
    )]
    #[cfg_attr(feature = "serde", serde(default))]
    auto_detach: bool,
    /// The delay in ms before the servo can detach itself in auto-detach mode (default: 20000ms).
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "crate::utils::is_default")
    )]
    #[cfg_attr(feature = "serde", serde(default))]
    detach_delay: usize,

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
    /// Inner handler to the task running the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    animation: Arc<Option<Animation>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    last_move: Arc<RwLock<Option<SystemTime>>>,
}

impl Servo {
    pub fn new(board: &Board, pin: u16, default: u16) -> Result<Self, Error> {
        Self::create(board, pin, default, false)
    }
    pub fn new_inverted(board: &Board, pin: u16, default: u16) -> Result<Self, Error> {
        Self::create(board, pin, default, true)
    }

    pub fn create(board: &Board, pin: u16, default: u16, inverted: bool) -> Result<Self, Error> {
        let pwm_range = Range::from([600, 2400]);

        let mut servo = Self {
            pin,
            state: Arc::new(RwLock::new(default)),
            default,
            servo_type: ServoType::default(),
            range: Range::from([0, 180]),
            pwm_range,
            degree_range: Range::from([0, 180]),
            inverted,
            auto_detach: false,
            detach_delay: 20000,
            previous: u16::MAX, // Ensure previous out-of-range: forces default at start
            protocol: board.get_protocol(),
            interval: Arc::new(None),
            animation: Arc::new(None),
            last_move: Arc::new(RwLock::new(None)),
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
        // Stops any animation running.
        self.stop();

        self.set_state(to.into())?;
        Ok(self)
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

    /// Sets the servo type.
    pub fn set_type(mut self, servo_type: ServoType) -> Self {
        self.servo_type = servo_type;
        self
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

    /// Retrieves the theoretical range of degrees of movement for the servo (some servos can range from 0 to 90°, 180°, 270°, 360°, etc...).
    pub fn get_degree_range(&self) -> Range<u16> {
        self.degree_range
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

    /// Retrieves the theoretical range of pwm controls the servo response to.
    pub fn get_pwn_range(&self) -> Range<u16> {
        self.pwm_range
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

    /// Retrieves if the servo command is set to be inverted.
    pub fn is_inverted(&self) -> bool {
        self.inverted
    }

    /// Sets the servo command inversion mode.
    pub fn set_inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Retrieves if the servo command is set to be auto_detach itself.
    pub fn is_auto_detach(&self) -> bool {
        self.auto_detach
    }

    /// Sets the servo to auto-detach itself after a given delay.
    pub fn set_auto_detach(mut self, auto_detach: bool) -> Self {
        self.auto_detach = match auto_detach {
            false => false,
            true => {
                match self.last_move.read().as_ref() {
                    None => {
                        self.protocol
                            .set_pin_mode(self.pin, PinModeId::OUTPUT)
                            .unwrap();
                    }
                    Some(last_move) => {
                        if last_move.elapsed().unwrap().as_millis() > (self.detach_delay as u128) {
                            self.protocol
                                .set_pin_mode(self.pin, PinModeId::OUTPUT)
                                .unwrap();
                        }
                    }
                }
                true
            }
        };
        self
    }

    /// Retrieves the delay (in ms) before the servo is auto-detach (if enabled).
    pub fn get_detach_delay(&self) -> usize {
        self.detach_delay
    }

    /// Sets the delay (in ms) before the servo is auto-detach (if enabled).
    pub fn set_detach_delay(mut self, detach_delay: usize) -> Self {
        self.detach_delay = detach_delay;
        self
    }
}

impl Display for Servo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SERVO (pin={}) [state={}, default={}, range={}-{}]",
            self.pin,
            self.state.read(),
            self.default,
            self.range.start,
            self.range.end
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for Servo {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Actuator for Servo {
    fn animate<S: Into<State>>(&mut self, state: S, duration: u64, transition: Easing) {
        let mut animation = Animation::from(
            Track::new(self.clone())
                .with_keyframe(Keyframe::new(state, 0, duration).set_transition(transition)),
        );
        animation.play();
        self.animation = Arc::new(Some(animation));
    }

    /// Stops the servo.
    /// Any animation running will be stopped after the current running step is executed.
    /// Any simple move running will be stopped at end position.
    fn stop(&mut self) {
        match &self.interval.as_ref() {
            None => {}
            Some(handler) => handler.abort(),
        };
    }

    /// Update the Servo position.
    fn set_state(&mut self, state: State) -> Result<State, Error> {
        // Convert from state.
        let value = match state {
            State::Integer(value) => Ok(value as u16),
            State::Signed(value) => match value >= 0 {
                true => Ok(value as u16),
                false => Err(Error::from(StateError)),
            },
            _ => Err(Error::from(StateError)),
        }?;

        // Clamp the request within the Servo range.
        let value: u16 = value.clamp(self.range.start, self.range.end);
        // No need to move if last move was already that one.
        // if state == self.previous {
        //     return Ok(state);
        // }

        let pwm: f64 = match self.inverted {
            false => value.scale(
                self.degree_range.start,
                self.degree_range.end,
                self.pwm_range.start,
                self.pwm_range.end,
            ),
            true => value.scale(
                self.degree_range.end,
                self.degree_range.start,
                self.pwm_range.start,
                self.pwm_range.end,
            ),
        };

        // Attach the pinMode if we are auto-detach mode.
        match self.auto_detach {
            false => self.protocol.analog_write(self.pin, pwm as u16)?,
            true => {
                self.protocol.set_pin_mode(self.pin, PinModeId::SERVO)?;
                self.protocol.analog_write(self.pin, pwm as u16)?;
                *self.last_move.write() = Some(SystemTime::now());
                let mut self_clone = self.clone();
                task::run(async move {
                    pause!(self_clone.detach_delay);
                    match self_clone.last_move.read().as_ref() {
                        None => {
                            debug!("Detach servo pin: {}", self_clone.pin);
                            self_clone
                                .protocol
                                .set_pin_mode(self_clone.pin, PinModeId::OUTPUT)
                                .unwrap();
                        }
                        Some(last_move) => {
                            if last_move.elapsed().unwrap().as_millis()
                                > (self_clone.detach_delay as u128)
                            {
                                debug!("Detach servo pin: {}", self_clone.pin);
                                self_clone
                                    .protocol
                                    .set_pin_mode(self_clone.pin, PinModeId::OUTPUT)
                                    .unwrap();
                            }
                        }
                    }
                })?;
            }
        }
        let current = self.state.read().clone();
        self.previous = current;
        *self.state.write() = value;
        Ok(value.into())
    }

    /// Retrieves the actuator current state.
    fn get_state(&self) -> State {
        self.state.read().clone().into()
    }

    /// Retrieves the actuator default (or neutral) state.
    fn get_default(&self) -> State {
        self.default.into()
    }

    /// Indicates the busy status, ie if the device is running an animation.
    fn is_busy(&self) -> bool {
        self.interval.is_some()
    }
}

// impl Drop for Servo {
//     fn drop(&mut self) {
//         let _ = self.set_state(self.get_default());
//     }
// }
