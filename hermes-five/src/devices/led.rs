use std::fmt::{Display, Formatter};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::animation::{Animation, Keyframe, Segment, Track};
use crate::board::Board;
use crate::devices::{Actuator, Device};
use crate::errors::{Error, StateError};
use crate::errors::HardwareError::IncompatibleMode;
use crate::protocols::{Pin, PinMode, PinModeId, Protocol};
use crate::utils::{Easing, State};
use crate::utils::scale::Scalable;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Led {
    // ########################################
    // # Basics
    /// The pin (id) of the board [`Board`] used to control the LED.
    pin: u16,
    /// The current LED state.
    #[cfg_attr(feature = "serde", serde(with = "crate::devices::arc_rwlock_serde"))]
    state: Arc<RwLock<u16>>,
    /// The LED default value (default: 0 - OFF).
    default: u16,

    // ########################################
    // # Settings
    /// Indicates the current LED brightness when ON.
    brightness: u16,

    // ########################################
    // # Volatile utility data.
    /// If the pin can do PWM, we store that mode here (memoization use only).
    #[cfg_attr(feature = "serde", serde(skip))]
    pwm_mode: Option<PinMode>,
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol: Box<dyn Protocol>,
    /// Inner handler to the task running the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    animation: Arc<Option<Animation>>,
}

impl Led {
    /// Creates an instance of a LED attached to a given board.
    ///
    /// # Parameters
    /// * `board`: the board which the LED is attached to
    /// * `pin`: the pin used to control the LED
    /// * `default`: the default LED state (boolean on/off)
    ///
    /// # Errors
    /// * `UnknownPin`: this function will bail an error if the LED pin does not exist for this board.
    /// * `IncompatibleMode`: this function will bail an error if the LED pin does not support OUTPUT or PWM mode.
    pub fn new(board: &Board, pin: u16, default: bool) -> Result<Self, Error> {
        let mut protocol = board.get_protocol();

        // Get the hardware corresponding pin.
        let hardware_pin = {
            let hardware = protocol.get_hardware().read();
            hardware.get_pin(pin)?.clone()
        };

        // Get the PWM mode if any
        let pwm_mode = hardware_pin.supports_mode(PinModeId::PWM);

        // Set pin mode to OUTPUT/PWM and compute default value accordingly.
        let pin_mode = match pwm_mode {
            None => PinModeId::OUTPUT,
            Some(_) => PinModeId::PWM,
        };
        protocol.set_pin_mode(pin, pin_mode)?;

        // Compute default value accordingly: 0 or 255 (max brightness).
        let default = match default {
            false => 0,
            true => 0xFF,
        };

        Ok(Self {
            pin,
            state: Arc::new(RwLock::new(default)),
            default,
            brightness: 0xFF,
            pwm_mode,
            protocol,
            animation: Arc::new(None),
        })
    }

    /// Turn the LED on.
    pub fn on(&mut self) -> Result<&Self, Error> {
        self.set_state(State::Integer(self.brightness as u64))?;
        Ok(self)
    }

    /// Turn the LED off.
    pub fn off(&mut self) -> Result<&Self, Error> {
        self.set_state(State::Integer(0))?;
        Ok(self)
    }

    /// Toggle the current state, if on then turn off, if off then turn on.
    pub fn toggle(&mut self) -> Result<&Self, Error> {
        match self.is_on() {
            true => self.off(),
            false => self.on(),
        }
    }

    /// Blink the LED on/off in phases of ms (milliseconds) duration.
    /// This is an animation and can be stopped by calling [`Led::stop()`].
    ///
    /// # Parameters
    /// * `ms`: the blink duration in milliseconds
    pub fn blink(&mut self, ms: u64) -> &Self {
        let mut animation = Animation::from(
            Segment::from(
                Track::new(self.clone())
                    .with_keyframe(Keyframe::new(true, 0, ms))
                    .with_keyframe(Keyframe::new(false, ms, ms * 2)),
            )
            .set_repeat(true),
        );
        animation.play();
        self.animation = Arc::new(Some(animation));

        self
    }

    /// Pulses the LED on/off (using fading) in phases of ms (milliseconds) duration.
    /// This is an animation and can be stopped by calling [`Led::stop()`].
    ///
    /// # Parameters
    /// * `ms`: the blink duration in milliseconds
    pub fn pulse(&mut self, ms: u64) -> &Self {
        let mut animation = Animation::from(
            Segment::from(
                Track::new(self.clone())
                    .with_keyframe(Keyframe::new(0xFFu16, 0, ms))
                    .with_keyframe(Keyframe::new(0u16, ms, ms * 2)),
            )
            .set_repeat(true),
        );
        animation.play();
        self.animation = Arc::new(Some(animation));

        self
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

    /// Retrieves the LED current brightness in percentage (0-100%).
    pub fn get_brightness(&self) -> u8 {
        match self.pwm_mode {
            None => 100,
            // Compute the brightness percentage (depending on resolution (255 on arduino for instance)).
            Some(pwm_mode) => {
                self.brightness
                    .scale(0, pwm_mode.get_max_possible_value() as u16, 0, 100)
            }
        }
    }

    /// Set the LED brightness (integer between 0-100) in percent of the max brightness. If a number
    /// higher than 100 is used, the brightness is set to 100%.
    /// If the requested brightness is 100%, the LED will reset to simple on/off (OUTPUT) mode.
    ///
    /// # Parameters
    /// * `brightness`: the requested brightness (between 0-100%)
    ///
    /// # Errors
    /// * `IncompatibleMode`: this function will bail an error if the LED pin does not support PWM.
    pub fn set_brightness(mut self, brightness: u8) -> Result<Self, Error> {
        // Brightness can only be between 0 and 100%
        let brightness = brightness.clamp(0, 100) as u16;

        // If the LED can use pwm mode: update the brightness
        let pwm_mode = self.pwm_mode.ok_or(IncompatibleMode {
            mode: PinModeId::PWM,
            pin: self.pin,
            context: "set LED brightness",
        })?;

        // Compute the brightness value (depending on resolution (255 on arduino for instance))
        let brightness = brightness.scale(0, 100, 0, pwm_mode.get_max_possible_value() as u16);

        // Sets the brightness.
        self.brightness = brightness;

        // If the value is higher than the brightness, we update it on the spot.
        if self.state.read().ne(&brightness) {
            self.set_state(State::Integer(brightness as u64))?;
        }

        Ok(self)
    }

    /// Indicates the LED current ON/OFF status.
    pub fn is_on(&self) -> bool {
        self.state.read().gt(&0)
    }
}

impl Display for Led {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LED (pin={}) [state={}, default={}, brightness={}]",
            self.pin,
            self.state.read(),
            self.default,
            self.brightness
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for Led {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Actuator for Led {
    /// Animates the LED
    fn animate<S: Into<State>>(&mut self, state: S, duration: u64, transition: Easing) {
        let mut animation = Animation::from(
            Track::new(self.clone())
                .with_keyframe(Keyframe::new(state, 0, duration).set_transition(transition)),
        );
        animation.play();
        self.animation = Arc::new(Some(animation));
    }

    /// Stops the current animation. This does not necessarily turn off the LED;
    /// it will remain in its current state when stopped.
    fn stop(&mut self) {
        if let Some(animation) = Arc::get_mut(&mut self.animation).and_then(Option::as_mut) {
            animation.stop();
        }
        self.animation = Arc::new(None);
    }

    /// Internal only: Update the LED to the target state.
    ///
    /// /!\ You should rather use [`Led::on()`], [`Led::off()`], [`Led::set_brightness()`]` functions.`
    fn set_state(&mut self, state: State) -> Result<State, Error> {
        let value = match state {
            State::Boolean(value) => match value {
                true => Ok(self.brightness),
                false => Ok(0),
            },
            State::Integer(value) => Ok(value as u16),
            _ => Err(Error::from(StateError)),
        }?;

        match self.get_pin_info()?.mode.id {
            // on/off digital operation.
            PinModeId::OUTPUT => self.protocol.digital_write(self.pin, value > 0),
            // pwm (brightness) mode.
            PinModeId::PWM => self.protocol.analog_write(self.pin, value),
            id => Err(Error::from(IncompatibleMode {
                mode: id,
                pin: self.pin,
                context: "update LED",
            })),
        }?;
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
        self.animation.is_some()
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::mocks::protocol::MockProtocol;
    use crate::pause;

    use super::*;

    fn _setup_led(pin: u16) -> Led {
        let board = Board::from(MockProtocol::default()); // Assuming a mock Board implementation
        Led::new(&board, pin, false).unwrap()
    }

    #[test]
    fn test_led_creation() {
        let led = _setup_led(13);
        assert_eq!(led.get_pin(), 13); // Ensure the correct pin is set
        assert_eq!(*led.state.read(), 0); // Initial state should be 0 (OFF)
        assert_eq!(led.brightness, 0xFF); // Default brightness should be 255
    }

    #[test]
    fn test_turn_on() {
        let mut led = _setup_led(13);
        assert!(led.on().is_ok()); // Turn LED on
        assert_eq!(*led.state.read(), 0xFF); // State should reflect the brightness (255)
    }

    #[test]
    fn test_turn_off() {
        let mut led = _setup_led(13);
        led.on().unwrap(); // Turn LED on first
        assert!(led.off().is_ok()); // Turn LED off
        assert_eq!(*led.state.read(), 0); // State should be 0 (OFF)
    }

    #[test]
    fn test_toggle() {
        let mut led = _setup_led(13);
        assert!(led.toggle().is_ok()); // Toggle to ON
        assert_eq!(*led.state.read(), 0xFF); // Should be ON (255)
        assert!(led.toggle().is_ok()); // Toggle to OFF
        assert_eq!(*led.state.read(), 0); // Should be OFF (0)
    }

    #[test]
    fn test_set_state() {
        let mut led = _setup_led(13);
        assert!(led.set_state(State::Boolean(true)).is_ok());
        assert_eq!(*led.state.read(), 0xFF); // State should reflect the brightness (100% = 255)
        assert!(led.set_state(State::Boolean(false)).is_ok());
        assert_eq!(*led.state.read(), 0x00); // Should be OFF (0)
        assert!(led
            .set_state(State::String(String::from("incorrect format")))
            .is_err()); // Should return an error due to incompatible state
                        // Force an incompatible pin mode
        let _ = led.protocol.set_pin_mode(led.pin, PinModeId::UNSUPPORTED);
        assert!(led.set_state(State::Boolean(false)).is_err()); // Should return an error due to incompatible pin mode.
    }

    #[test]
    fn test_brightness_calculation() {
        let mut led = _setup_led(8);

        // Force custom pinMode on 10bits
        led.pwm_mode = Some(PinMode {
            id: Default::default(),
            resolution: 10,
        });

        // Check brightness at 0%
        let led = led.set_brightness(0).unwrap();
        assert_eq!(led.get_brightness(), 0);
        assert_eq!(led.brightness, 0);
        assert_eq!(*led.state.read(), 0);

        // Check brightness at 50%
        let led = led.set_brightness(50).unwrap();
        assert_eq!(led.get_brightness(), 50);
        assert_eq!(led.brightness, 512);
        assert_eq!(*led.state.read(), 512);

        // Check brightness at 100%
        let led = led.set_brightness(100).unwrap();
        assert_eq!(led.get_brightness(), 100);
        assert_eq!(led.brightness, 1023);
        assert_eq!(*led.state.read(), 1023);

        // Check brightness at 120%
        let led = led.set_brightness(120).unwrap();
        assert_eq!(led.get_brightness(), 100);
        assert_eq!(led.brightness, 1023);
        assert_eq!(*led.state.read(), 1023);
    }

    #[test]
    fn test_set_brightness_valid() {
        let result = _setup_led(8).set_brightness(50);
        assert!(result.is_ok()); // Set brightness to 50%
        let mut led = result.unwrap();

        assert_eq!(led.get_brightness(), 50); // Check the brightness is correctly set
        assert_eq!(led.brightness, 128); // 50% of 255
        assert_eq!(*led.state.read(), 128); // State should reflect the brightness (50%)

        assert_eq!(led.get_brightness(), 50); // Check the brightness is correctly set
        assert_eq!(led.brightness, 128); // 50% of 255
        assert_eq!(*led.state.read(), 128); // State should reflect the brightness (50%)

        assert!(led.set_state(State::Boolean(false)).is_ok());
        assert_eq!(*led.state.read(), 0x00);
        assert!(led.set_state(State::Boolean(true)).is_ok());
        assert_eq!(*led.state.read(), 128); // State should reflect the brightness (50%)
    }

    #[test]
    fn test_set_brightness_incompatible_mode() {
        let led = _setup_led(13);
        assert_eq!(led.get_brightness(), 100);
        let result = led.set_brightness(50);
        assert!(result.is_err()); // Should return an error due to incompatible mode
    }

    #[test]
    fn test_default_value() {
        let led = _setup_led(13);
        assert_eq!(led.get_state().as_integer(), 0); // Should be full OFF by default.
        let led = Led::new(&Board::from(MockProtocol::default()), 13, true).unwrap(); // Setup with default value TRUE
        assert_eq!(led.get_default().as_integer(), 0xFF); // Default should be fully ON (255).
        assert_eq!(led.get_state().as_integer(), 0xFF); // State should be equal to default.
    }

    #[test]
    fn test_get_pin_info() {
        let led = _setup_led(13);
        let pin_info = led.get_pin_info();
        assert!(pin_info.is_ok()); // Ensure that pin information retrieval is successful
    }

    #[hermes_macros::test]
    fn test_led_blink() {
        let mut led = _setup_led(13);
        assert!(!led.is_busy());
        led.stop(); // Stop something not started should not fail.
        led.blink(50); // Set a blink interval of 50 ms
        pause!(100);
        assert!(led.is_busy()); // Animation is currently running.
        led.stop();
        assert!(!led.is_busy());
    }

    #[hermes_macros::test]
    fn test_led_pulse() {
        let mut led = _setup_led(8);
        assert!(!led.is_busy());
        led.stop(); // Stop something not started should not fail.
        led.pulse(50); // Set a fading pulse interval of 50 ms
        pause!(100);
        assert!(led.is_busy()); // Animation is currently running.
        led.stop();
        assert!(!led.is_busy());
    }

    #[hermes_macros::test]
    fn test_animation() {
        let mut led = _setup_led(8);
        assert!(!led.is_busy());
        // Stop something not started should not fail.
        led.stop();
        // Fade in the LED to brightness
        led.animate(led.get_brightness(), 500, Easing::Linear);
        pause!(100);
        assert!(led.is_busy()); // Animation is currently running.
        led.stop();
    }

    #[test]
    fn test_is_on() {
        let mut led = _setup_led(13);
        assert!(!led.is_on()); // Initially the LED is off
        led.on().unwrap();
        assert!(led.is_on()); // After turning on, the LED should be on
    }

    #[test]
    fn test_display_impl() {
        let led = _setup_led(13);
        let display_str = format!("{}", led);
        assert!(display_str.contains("LED (pin=13)")); // Check that the pin is correctly displayed
    }
}
