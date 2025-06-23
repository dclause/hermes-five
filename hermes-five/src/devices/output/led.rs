use crate::animations::{Animation, Keyframe, Segment, Track};
use crate::devices::OutputDevice;
use crate::errors::HardwareError::IncompatiblePin;
use crate::errors::{Error, StateError};
use crate::hardware::Hardware;
use crate::io::{IoProtocol, Pin, PinMode, PinModeId};
use crate::utils::{Scalable, State};
use hermes_five_macros::output_device;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

/// Represents a LED connected to a digital or PWM-capable pin.
///
/// This struct provides high-level control over an LED, abstracting both
/// simple on/off digital control and variable brightness via PWM.
///
/// # Pin Compatibility
/// - `PinMode::Output`: Supports binary on/off control.
/// - `PinMode::Pwm`: Enables brightness control via analog-style output.
///
/// # Behavior
/// - Automatically configures the pin during setup.
/// - Internally stores LED state atomically for thread-safe updates.
/// - Supports scaling, animation tracks, and optional sink mode (if configured).
///
/// # Errors
/// Creating or operating on an LED may fail if:
/// - The pin mode is incompatible (e.g., analog-only pin).
/// - The hardware backend rejects pin configuration.
/// - The LED is used in a state-inconsistent way.
///
/// # Features
/// - (Optional) Serde serialization if `serde` feature is enabled.
///
/// Use `Led::new` or `Led::new_sink` for setup depending on circuit configuration.
#[output_device(u16)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Led {
    // ########################################
    // # Basics
    /// The pin (id) of the [`Board`] used to control the LED.
    pin: u8,
    /// The current LED state.
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::arc_atomic_serde"))]
    state: Arc<AtomicU16>,
    /// Activate led sink mode (ie cathode is plugged to the board)
    is_sink: bool,

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
    protocol: Box<dyn IoProtocol>,
}

impl Led {
    /// Creates a LED instance attached to the specified pin on the board using source mode.
    ///
    /// In source mode, the board pin is configured as an output that provides current (+5V) to the LED.
    /// The LED anode (+) connects to the board pin through a current-limiting resistor,
    /// and the cathode (-) connects to GND.
    ///
    /// To turn the LED on, the pin is driven HIGH (+5V).
    ///
    /// # Errors
    /// - `UnknownPin`: returned if the specified pin does not exist on the board.
    /// - `IncompatibleMode`: returned if the pin does not support OUTPUT or PWM mode.
    pub fn new(board: &dyn Hardware, pin: u8, default: bool) -> Result<Self, Error> {
        let mut protocol = board.get_protocol();

        // Get the hardware corresponding pin.
        let hardware_pin = {
            let hardware = protocol.get_io().read();
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

        let mut led = Self {
            pin,
            state: Arc::new(AtomicU16::new(default)),
            default,
            is_sink: false,
            brightness: 0xFF,
            pwm_mode,
            protocol,
            animation: Arc::new(None),
        };

        led.reset()?;

        Ok(led)
    }

    /// Creates a LED instance attached to the specified pin on the board using sink mode.
    ///
    /// In sink mode, the board pin is configured as an output that sinks current to GND.
    /// The LED cathode (-) connects to +5V through a current-limiting resistor,
    /// and the anode (+) connects to the board pin.
    ///
    /// To turn the LED on, the pin is driven LOW (GND).
    ///
    /// # Errors
    /// - `UnknownPin`: returned if the specified pin does not exist on the board.
    /// - `IncompatibleMode`: returned if the pin does not support OUTPUT or PWM mode.
    pub fn new_sink(board: &dyn Hardware, pin: u8, default: bool) -> Result<Self, Error> {
        let mut led = Led::new(board, pin, !default)?;
        led.is_sink = true;
        Ok(led)
    }

    /// Turns the LED on.
    pub fn turn_on(&mut self) -> Result<&Self, Error> {
        self.set_state(State::Integer(self.brightness as u64))?;
        Ok(self)
    }

    /// Turns the LED off.
    pub fn turn_off(&mut self) -> Result<&Self, Error> {
        self.set_state(State::Integer(0))?;
        Ok(self)
    }

    /// Toggles the current state, if on then turn off, if off then turn on.
    pub fn toggle(&mut self) -> Result<&Self, Error> {
        match self.is_on() {
            true => self.turn_off(),
            false => self.turn_on(),
        }
    }

    /// Blinks the LED on/off in phases of milliseconds duration.
    /// This is an animation and can be stopped by calling [`Led::stop()`].
    pub fn blink(&mut self, ms: u64) -> &Self {
        self.stop();
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
    pub fn pulse(&mut self, ms: u64) -> &Self {
        self.stop();
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
    // Getters.

    /// Returns the pin (id) used by the device.
    pub fn get_pin(&self) -> u8 {
        self.pin
    }

    /// Returns the [`Pin`] information.
    pub fn get_pin_info(&self) -> Result<Pin, Error> {
        let lock = self.protocol.get_io().read();
        Ok(lock.get_pin(self.pin)?.clone())
    }

    /// Returns the LED current brightness in percentage (0-100%).
    pub fn get_brightness(&self) -> u8 {
        match self.pwm_mode {
            None => 100,
            // Compute the brightness percentage (depending on resolution (255 on arduino for instance)).
            Some(pwm_mode) => self
                .brightness
                .scale(0, pwm_mode.get_max_possible_value(), 0, 100),
        }
    }

    /// Sets the LED brightness as an integer percentage from 0 to 100.
    ///
    /// Values above 100 are clamped to 100.
    ///
    /// # Errors
    /// * `IncompatiblePin`: returned if the LED pin does not support PWM.
    pub fn set_brightness(&mut self, brightness: u8) -> Result<&Self, Error> {
        // Brightness can only be between 0 and 100%
        let brightness = brightness.clamp(0, 100) as u16;

        // If the LED can use pwm mode: update the brightness
        let pwm_mode = self.pwm_mode.ok_or(IncompatiblePin {
            mode: PinModeId::PWM,
            pin: self.pin,
            context: "set LED brightness",
        })?;

        // Compute the brightness value (depending on resolution (255 on arduino for instance))
        let brightness = brightness.scale(0, 100, 0, pwm_mode.get_max_possible_value());

        // Sets the brightness.
        self.brightness = brightness;

        // If the value is higher than the brightness, we update it on the spot.
        if self.get_value().ne(&self.brightness) {
            self.set_state(State::Integer(self.brightness as u64))?;
        }

        Ok(self)
    }

    /// Indicates if the LED is current ON (regardless its brightness).
    pub fn is_on(&self) -> bool {
        self.get_value().gt(&0)
    }

    /// Indicates if the LED is current OFF.
    pub fn is_off(&self) -> bool {
        self.get_value().eq(&0)
    }

    /// Converts the input `State` into a brightness value (u16).
    /// For sink mode, the brightness is inverted by mapping [0..255] to [255..0]
    /// using the `scale` function.
    ///
    /// # Errors
    /// Returns `StateError` if the state variant is unsupported.
    #[inline(always)]
    fn parse_state(&self, state: State) -> Result<u16, Error> {
        let value = match state {
            State::Boolean(value) => match value {
                true => Ok(self.brightness),
                false => Ok(0),
            },
            State::Integer(value) => Ok(value as u16),
            State::Float(value) => Ok(value as u16),
            State::Signed(value) => Ok(value.max(0) as u16),
            _ => Err(StateError),
        }?;

        // Reverse the value if sink mode.
        let value = if self.is_sink {
            value.scale(0, 0xFF, 0xFF, 0)
        } else {
            value
        };

        Ok(value)
    }

    /// Applies the given value to the LED hardware pin according to its configured mode.
    ///
    /// This function checks the pin mode:
    /// - If the pin mode is `OUTPUT`, it performs a digital write:
    ///   the LED is turned ON if `value` > 0, OFF otherwise.
    /// - If the pin mode is `PWM`, it performs an analog write with the provided `value`,
    ///   controlling the LED brightness.
    /// - If the pin mode is neither `OUTPUT` nor `PWM`, returns an `IncompatiblePin` error.
    ///
    /// # Parameters
    /// * `value`: The brightness or ON/OFF state value to apply to the LED.
    ///
    /// # Errors
    /// Returns `IncompatiblePin` if the pin mode does not support digital or PWM output.
    #[inline(always)]
    fn apply_value(&mut self, value: u16) -> Result<(), Error> {
        match self.get_pin_info()?.mode.id {
            // on/off digital operation.
            PinModeId::OUTPUT => self.protocol.digital_write(self.pin, value > 0),
            // pwm (brightness) mode.
            PinModeId::PWM => self.protocol.analog_write(self.pin, value),
            id => Err(Error::from(IncompatiblePin {
                mode: id,
                pin: self.pin,
                context: "update LED",
            })),
        }
    }

    #[inline(always)]
    fn get_value(&self) -> u16 {
        self.state.load(Ordering::Relaxed)
    }
    #[inline(always)]
    fn set_value(&self, value: u16) {
        self.state.store(value, Ordering::SeqCst)
    }
}

impl Display for Led {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LED (pin={}) [mode={}, state={}, default={}, brightness={}, animating={}]",
            self.pin,
            self.get_pin_info()
                .map_or("unknown".to_string(), |p| format!("{:?}", p.mode.id)),
            self.get_value(),
            self.default,
            self.brightness,
            self.is_busy()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::animations::Easing;
    use crate::hardware::Board;
    use crate::mocks::MockProtocol;
    use crate::pause;

    use super::*;

    fn _setup_led(pin: u8) -> Led {
        let board = Board::new(MockProtocol::default()); // Assuming a mock Board implementation
        Led::new(&board, pin, false).unwrap()
    }

    #[test]
    fn test_led_creation() {
        let led = _setup_led(13);
        assert_eq!(led.get_pin(), 13); // Ensure the correct pin is set
        assert_eq!(led.get_value(), 0); // Initial state should be 0 (OFF)
        assert_eq!(led.brightness, 0xFF); // Default brightness should be 255
    }

    #[test]
    fn test_turn_on() {
        let mut led = _setup_led(13);
        assert!(led.turn_on().is_ok()); // Turn LED on
        assert_eq!(led.get_value(), 0xFF); // State should reflect the brightness (255)
    }

    #[test]
    fn test_turn_off() {
        let mut led = _setup_led(13);
        led.turn_on().unwrap(); // Turn LED on first
        assert!(led.turn_off().is_ok()); // Turn LED off
        assert_eq!(led.get_value(), 0); // State should be 0 (OFF)
    }

    #[test]
    fn test_toggle() {
        let mut led = _setup_led(13);
        assert!(led.toggle().is_ok()); // Toggle to ON
        assert_eq!(led.get_value(), 0xFF); // Should be ON (255)
        assert!(led.toggle().is_ok()); // Toggle to OFF
        assert_eq!(led.get_value(), 0); // Should be OFF (0)
    }

    #[test]
    fn test_set_state() {
        let mut led = _setup_led(13);

        assert!(led.set_state(State::Boolean(true)).is_ok());
        assert_eq!(led.get_value(), 0xFF); // State should reflect the brightness (100% = 255)
        assert!(led.set_state(State::Boolean(false)).is_ok());
        assert_eq!(led.get_value(), 0x00); // Should be OFF (0)

        assert!(led.set_state(State::Integer(50)).is_ok());
        assert_eq!(led.get_value(), 50);
        assert!(led.set_state(State::Float(60.0)).is_ok());
        assert_eq!(led.get_value(), 60);
        assert!(led.set_state(State::Signed(70)).is_ok());
        assert_eq!(led.get_value(), 70);
        assert!(led.set_state(State::Signed(-70)).is_ok());
        assert_eq!(led.get_value(), 0);

        // Incorrect state type.
        assert!(led
            .set_state(State::String(String::from("incorrect format")))
            .is_err()); // Should return an error due to incompatible state
                        // Force an incompatible pin mode

        // Incorrect pin type.
        let _ = led.protocol.set_pin_mode(led.pin, PinModeId::UNSUPPORTED);
        assert!(led.set_state(State::Boolean(true)).is_err()); // Should return an error due to incompatible pin mode.
    }

    #[test]
    fn test_sink_set_state() {
        let board = Board::new(MockProtocol::default()); // Assuming a mock Board implementation
        let mut led = Led::new_sink(&board, 13, false).unwrap();

        assert!(led.set_state(State::Boolean(true)).is_ok());
        assert_eq!(led.get_value(), 0x00); // State should reflect the brightness (100% = 255)
        assert!(led.set_state(State::Boolean(false)).is_ok());
        assert_eq!(led.get_value(), 0xFF); // Should be OFF (0)

        assert!(led.set_state(State::Integer(50)).is_ok());
        assert_eq!(led.get_value(), 205);
        assert!(led.set_state(State::Float(60.0)).is_ok());
        assert_eq!(led.get_value(), 195);
        assert!(led.set_state(State::Signed(70)).is_ok());
        assert_eq!(led.get_value(), 185);
        assert!(led.set_state(State::Signed(-70)).is_ok());
        assert_eq!(led.get_value(), 0xFF);

        // Incorrect state type.
        assert!(led
            .set_state(State::String(String::from("incorrect format")))
            .is_err()); // Should return an error due to incompatible state
                        // Force an incompatible pin mode

        // Incorrect pin type.
        let _ = led.protocol.set_pin_mode(led.pin, PinModeId::UNSUPPORTED);
        assert!(led.set_state(State::Boolean(true)).is_err()); // Should return an error due to incompatible pin mode.
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
        assert!(led.set_brightness(0).is_ok());
        assert_eq!(led.get_brightness(), 0);
        assert_eq!(led.brightness, 0);
        assert_eq!(led.get_value(), 0);

        // Check brightness at 50%
        assert!(led.set_brightness(50).is_ok());
        assert_eq!(led.get_brightness(), 50);
        assert_eq!(led.brightness, 512);
        assert_eq!(led.get_value(), 512);

        // Check brightness at 100%
        assert!(led.set_brightness(100).is_ok());
        assert_eq!(led.get_brightness(), 100);
        assert_eq!(led.brightness, 1023);
        assert_eq!(led.get_value(), 1023);

        // Check brightness at 120%
        assert!(led.set_brightness(120).is_ok());
        assert_eq!(led.get_brightness(), 100);
        assert_eq!(led.brightness, 1023);
        assert_eq!(led.get_value(), 1023);
    }

    #[test]
    fn test_set_brightness_valid() {
        let mut led = _setup_led(8);
        let result = led.set_brightness(50);
        assert!(result.is_ok()); // Set brightness to 50%

        assert_eq!(led.get_brightness(), 50); // Check the brightness is correctly set
        assert_eq!(led.brightness, 128); // 50% of 255
        assert_eq!(led.get_value(), 128); // State should reflect the brightness (50%)

        assert!(led.set_state(State::Boolean(false)).is_ok());
        assert_eq!(led.get_value(), 0x00);
        assert!(led.set_state(State::Boolean(true)).is_ok());
        assert_eq!(led.get_value(), 128); // State should reflect the brightness (50%)
    }

    #[test]
    fn test_set_brightness_incompatible_mode() {
        let mut led = _setup_led(13);
        assert_eq!(led.get_brightness(), 100);
        let result = led.set_brightness(50);
        assert!(result.is_err()); // Should return an error due to incompatible mode
    }

    #[test]
    fn test_default_value() {
        let led = _setup_led(13);
        assert_eq!(led.get_state().as_integer(), 0); // Should be full OFF by default.
        let led = Led::new(&Board::new(MockProtocol::default()), 13, true).unwrap(); // Setup with default value TRUE
        assert_eq!(led.get_default().as_integer(), 0xFF); // Default should be fully ON (255).
        assert_eq!(led.get_state().as_integer(), 0xFF); // State should be equal to default.
    }

    #[test]
    fn test_get_pin_info() {
        let led = _setup_led(13);
        let pin_info = led.get_pin_info();
        assert!(pin_info.is_ok()); // Ensure that pin information retrieval is successful
    }

    #[hermes_five_macros::test]
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

    #[hermes_five_macros::test]
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

    #[hermes_five_macros::test]
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
    fn test_is_on_off() {
        let mut led = _setup_led(13);
        assert!(!led.is_on()); // Initially the LED is off
        assert!(led.is_off());
        led.turn_on().unwrap();
        assert!(led.is_on()); // After turning on, the LED should be on
        assert!(!led.is_off());
    }

    #[hermes_five_macros::test]
    fn test_display_impl() {
        let mut led = _setup_led(13);
        let display_str = format!("{}", led);
        assert_eq!(
            display_str,
            "LED (pin=13) [mode=OUTPUT, state=0, default=0, brightness=255, animating=false]"
        );

        led.blink(200);
        let display_str = format!("{}", led);
        assert_eq!(
            display_str,
            "LED (pin=13) [mode=OUTPUT, state=0, default=0, brightness=255, animating=true]"
        );

        led.stop();
    }
}
