use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::SystemTime;

use parking_lot::RwLock;

use crate::animations::{Animation, Easing, Keyframe, Segment, Track};
use crate::devices::{Device, Output};
use crate::errors::HardwareError::IncompatibleMode;
use crate::errors::{Error, StateError};
use crate::hardware::Board;
use crate::io::{IoProtocol, Pin, PinModeId};
use crate::utils::{task, Range, Scalable, State};
use crate::{pause, pause_sync};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServoType {
    #[default]
    Standard,
    Continuous,
}

/// Represents a Servo controlled by a PWM pin.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Servo {
    // ########################################
    // # Basics
    /// The pin (id) of the [`Board`] used to control the Servo.
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
    protocol: Box<dyn IoProtocol>,
    /// Inner handler to the task running the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    animation: Arc<Option<Animation>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    last_move: Arc<RwLock<Option<SystemTime>>>,
}

impl Servo {
    /// Creates an instance of a Servo attached to a given board.
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the pin does not support SERVO mode.
    pub fn new(board: &Board, pin: u16, default: u16) -> Result<Self, Error> {
        Self::create(board, pin, default, false)
    }

    /// Creates an instance of an inverted Servo attached to a given board (see [`Self::set_inverted`]`).
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the pin does not support SERVO mode.
    pub fn new_inverted(board: &Board, pin: u16, default: u16) -> Result<Self, Error> {
        Self::create(board, pin, default, true)
    }

    /// Inner helper.
    fn create(board: &Board, pin: u16, default: u16, inverted: bool) -> Result<Self, Error> {
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
            animation: Arc::new(None),
            last_move: Arc::new(RwLock::new(None)),
        };

        // --
        // The following may seem tedious, but it ensures we attach the servo with the default value already set.
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

    /// Moves the servo to the requested position at max speed.
    pub fn to(&mut self, to: u16) -> Result<&Self, Error> {
        // Stops any animation running.
        self.stop();

        self.set_state(to.into())?;
        Ok(self)
    }

    /// Sweeps the servo in phases of ms (milliseconds) duration.
    /// This is an animation and can be stopped by calling [`Self::stop()`].
    ///
    /// # Parameters
    /// * `ms`: the blink duration in milliseconds
    pub fn sweep(&mut self, ms: u64) -> &Self {
        let mut animation = Animation::from(
            Segment::from(
                Track::new(self.clone())
                    .with_keyframe(Keyframe::new(self.range.end, 0, ms))
                    .with_keyframe(Keyframe::new(self.range.start, ms, ms * 2)),
            )
            .set_repeat(true),
        );
        animation.play();
        self.animation = Arc::new(Some(animation));

        self
    }

    // ########################################
    // Setters and Getters.

    /// Returns the pin (id) used by the device.
    pub fn get_pin(&self) -> u16 {
        self.pin
    }

    /// Returns [`Pin`] information.
    pub fn get_pin_info(&self) -> Result<Pin, Error> {
        let lock = self.protocol.get_data().read();
        Ok(lock.get_pin(self.pin)?.clone())
    }

    /// Returns the servo type.
    pub fn get_type(&self) -> ServoType {
        self.servo_type
    }

    /// Sets the servo type.
    pub fn set_type(mut self, servo_type: ServoType) -> Self {
        self.servo_type = servo_type;
        self
    }

    /// Returns the servo motion range limitation in degree.
    ///
    /// A servo has a physical range (cf [`Self::set_degree_range`]) corresponding to a command range
    /// limitation (cf [`Self::set_pwn_range`]). Those are intrinsic top the servo itself. On the contrary,
    /// the motion range limitation here is a limitation you want to set for your servo because of how
    /// it is used in your robot: for example an arm that can turn only 20-40° in motion range.
    pub fn get_range(&self) -> Range<u16> {
        self.range
    }

    /// Sets the Servo motion range limitation in degree. This guarantee the servo to stays in the given
    /// range at any time.
    ///
    /// - No matter the order given, the range will always have min <= max
    /// - No matter the values given, the range will always stay within the Servo `degree_range`.
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

    /// Returns  the theoretical range of degrees of movement for the servo (some servos can range from 0 to 90°, 180°, 270°, 360°, etc.).
    pub fn get_degree_range(&self) -> Range<u16> {
        self.degree_range
    }

    /// Sets the theoretical range of degrees of movement for the servo (some servos can range from 0 to 90°, 180°, 270°, 360°, etc.).
    ///
    /// - No matter the order given, the range will always have min <= max
    /// - This may impact the `range` since it will always stay within the given `degree_range`.
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

    /// Returns the theoretical range of pwm controls the servo response to.
    pub fn get_pwn_range(&self) -> Range<u16> {
        self.pwm_range
    }

    /// Sets the theoretical range of pwm controls the servo response to.
    ///
    /// # Parameters
    /// * `pwm_range`: the range limitation
    pub fn set_pwn_range<R: Into<Range<u16>>>(mut self, pwm_range: R) -> Result<Self, Error> {
        let input = pwm_range.into();
        self.pwm_range = input;
        self.protocol.servo_config(self.pin, input)?;
        Ok(self)
    }

    /// Returns if the servo command is set to be inverted.
    pub fn is_inverted(&self) -> bool {
        self.inverted
    }

    /// Sets the servo command inversion mode.
    pub fn set_inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Returns if the servo command is set to be auto_detach itself.
    pub fn is_auto_detach(&self) -> bool {
        self.auto_detach
    }

    /// Sets the servo to auto-detach itself after a given delay.
    pub fn set_auto_detach(mut self, auto_detach: bool) -> Self {
        self.auto_detach = match auto_detach {
            false => {
                self.protocol
                    .set_pin_mode(self.pin, PinModeId::SERVO)
                    .unwrap();
                false
            }
            true => {
                self.protocol
                    .set_pin_mode(self.pin, PinModeId::OUTPUT)
                    .unwrap();
                true
            }
        };
        self
    }

    /// Returns the delay (in ms) before the servo is auto-detach (if enabled).
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
impl Output for Servo {
    fn get_state(&self) -> State {
        (*self.state.read()).into()
    }
    /// Internal only: you should rather use [`Self::to()`] function.
    fn set_state(&mut self, state: State) -> Result<State, Error> {
        // Convert from state.
        let value = match state {
            State::Integer(value) => Ok(value as u16),
            State::Signed(value) => match value >= 0 {
                true => Ok(value as u16),
                false => Err(StateError),
            },
            State::Float(value) => match value >= 0.0 {
                true => Ok(value as u16),
                false => Err(StateError),
            },
            _ => Err(StateError),
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
                    if let Some(last_move) = self_clone.last_move.read().as_ref() {
                        if last_move.elapsed().unwrap().as_millis()
                            >= (self_clone.detach_delay as u128)
                        {
                            self_clone
                                .protocol
                                .set_pin_mode(self_clone.pin, PinModeId::OUTPUT)
                                .unwrap();
                        }
                    }
                })?;
            }
        }
        let current = *self.state.read();
        self.previous = current;
        *self.state.write() = value;
        Ok(value.into())
    }
    fn get_default(&self) -> State {
        self.default.into()
    }

    fn animate<S: Into<State>>(&mut self, state: S, duration: u64, transition: Easing) {
        let mut animation = Animation::from(
            Track::new(self.clone())
                .with_keyframe(Keyframe::new(state, 0, duration).set_transition(transition)),
        );
        animation.play();
        self.animation = Arc::new(Some(animation));
    }
    fn is_busy(&self) -> bool {
        self.animation.is_some()
    }
    fn stop(&mut self) {
        if let Some(animation) = Arc::get_mut(&mut self.animation).and_then(Option::as_mut) {
            animation.stop();
        }
        self.animation = Arc::new(None);
    }
}

#[cfg(test)]
mod tests {
    use crate::animations::Easing;
    use crate::devices::{Output, Servo};
    use crate::hardware::Board;
    use crate::io::PinModeId;
    use crate::mocks::plugin_io::MockIoProtocol;
    use crate::pause;
    use crate::utils::{Range, State};
    use hermes_five::devices::ServoType;

    fn _setup_servo(pin: u16) -> Servo {
        let board = Board::new(MockIoProtocol::default()); // Assuming a mock Board implementation
        Servo::new(&board, pin, 90).unwrap()
    }

    #[test]
    fn test_servo_creation() {
        let board = Board::new(MockIoProtocol::default());

        let servo = Servo::new(&board, 12, 90).unwrap();
        assert_eq!(servo.get_pin(), 12);
        assert_eq!(*servo.state.read(), 90);
        assert!(!servo.is_inverted());

        let inverted_servo = Servo::new_inverted(&board, 12, 90).unwrap();
        assert!(inverted_servo.is_inverted());

        let servo = Servo::new(&board, 12, 66).unwrap();
        assert_eq!(servo.get_default(), State::Integer(66));
        assert_eq!(servo.get_state(), State::Integer(66));
    }

    #[test]
    fn test_servo_move() {
        let mut servo = _setup_servo(12);
        let result = servo.to(150); // Move the servo to position 150.
        assert!(result.is_ok());
        assert_eq!(*servo.state.read(), 150); // Servo state should be updated to 150.
    }

    #[test]
    fn test_servo_range_setting() {
        let mut servo = _setup_servo(12);
        servo = servo.set_range([100, 200]); // Setting new range.
        assert_eq!(servo.get_range(), Range::from([100, 180])); // Ensure the range is updated but clamp in the theoretical degree_range.
        assert_eq!(servo.default, 100); // Default remains within the range.
    }

    #[test]
    fn test_servo_degree_range_setting() {
        let mut servo = _setup_servo(12);
        servo = servo.set_degree_range([100, 200]); // Setting new range.
        assert_eq!(servo.get_degree_range(), Range::from([100, 200]));
        assert_eq!(servo.get_range(), Range::from([100, 180])); // Ensure the range is updated/clamped in the theoretical degree_range.
        assert_eq!(servo.default, 100); // Default remains within the range.
    }

    #[test]
    fn test_servo_pwm_range_setting() {
        let servo = _setup_servo(12);
        let result = servo.set_pwn_range([999, 9999]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get_pwn_range(), Range::from([999, 9999]));
    }

    #[test]
    fn test_servo_detach_delay() {
        let mut servo = _setup_servo(12);
        assert_eq!(servo.get_detach_delay(), 20000);
        servo = servo.set_detach_delay(100);
        assert_eq!(servo.get_detach_delay(), 100);
    }

    #[hermes_macros::test]
    fn test_servo_auto_detach() {
        let mut servo = _setup_servo(12).set_auto_detach(true).set_detach_delay(300);
        assert!(servo.is_auto_detach());
        assert_eq!(servo.get_pin_info().unwrap().mode.id, PinModeId::OUTPUT);

        // Do not auto-detach: should reset pinMode to SERVO for proper use.
        servo = servo.set_auto_detach(false);
        assert!(!servo.is_auto_detach());
        assert_eq!(servo.get_pin_info().unwrap().mode.id, PinModeId::SERVO);
        let _ = servo.to(180);

        // Auto-detach unmoved servo: it should detach right away.
        servo = servo.set_auto_detach(true);
        assert_eq!(servo.get_pin_info().unwrap().mode.id, PinModeId::OUTPUT);
        assert!(servo.is_auto_detach());

        // Moving should auto-reattach.
        servo.to(180).expect("");
        assert_eq!(servo.get_pin_info().unwrap().mode.id, PinModeId::SERVO);
        pause!(80);
        // Continue moving should reset detach timer
        servo.to(180).expect("");
        pause!(80);
        assert_eq!(servo.get_pin_info().unwrap().mode.id, PinModeId::SERVO);
        // No move ultimately leads to auto-detaching
        pause!(3000);
        assert!(servo.is_auto_detach());
        assert_eq!(servo.get_pin_info().unwrap().mode.id, PinModeId::OUTPUT);
    }

    #[test]
    fn test_servo_type() {
        let mut servo = _setup_servo(12);
        assert_eq!(servo.get_type(), ServoType::Standard);
        servo = servo.set_type(ServoType::Continuous);
        assert_eq!(servo.get_type(), ServoType::Continuous);
    }

    #[test]
    fn test_servo_state() {
        let mut servo = _setup_servo(12);

        // Move in range
        assert!(servo.set_state(State::Integer(66)).is_ok());
        assert_eq!(servo.get_state(), State::Integer(66));

        // Move outside range
        assert!(servo.set_state(State::Integer(666)).is_ok());
        assert_eq!(servo.get_state(), State::Integer(180));

        // Move signed range
        assert!(servo.set_state(State::Signed(-12)).is_err());
        assert!(servo.set_state(State::Signed(12)).is_ok());
        assert_eq!(servo.get_state(), State::Integer(12));

        // Move float range
        assert!(servo.set_state(State::Float(-12.5)).is_err());
        assert!(servo.set_state(State::Float(12.5)).is_ok());
        assert_eq!(servo.get_state(), State::Integer(12));

        // Move unknown type
        assert!(servo.set_state(State::Boolean(true)).is_err());

        // Move inverted range
        let mut servo = _setup_servo(12).set_inverted(true);
        assert!(servo.set_state(State::Integer(66)).is_ok());
        assert_eq!(servo.get_state(), State::Integer(66));
    }

    #[hermes_macros::test]
    fn test_servo_sweep() {
        let mut servo = _setup_servo(12);
        assert!(!servo.is_busy());
        servo.stop();
        servo.sweep(200);
        pause!(100);
        assert!(servo.is_busy()); // Animation is currently running.
        servo.stop();
        assert!(!servo.is_busy());
    }

    #[hermes_macros::test]
    fn test_animation() {
        let mut servo = _setup_servo(12);
        assert!(!servo.is_busy());
        // Stop something not started should not fail.
        servo.stop();
        // Fade in the LED to brightness
        servo.animate(66, 500, Easing::Linear);
        pause!(100);
        assert!(servo.is_busy()); // Animation is currently running.
        servo.stop();
        assert!(!servo.is_busy());
    }

    #[test]
    fn test_servo_display() {
        let servo = _setup_servo(12);
        let display_output = format!("{}", servo);
        let expected_output = "SERVO (pin=12) [state=90, default=90, range=0-180]";
        assert_eq!(display_output, expected_output);
    }
}
