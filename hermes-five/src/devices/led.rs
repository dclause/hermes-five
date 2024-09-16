use std::fmt::{Display, Formatter};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::animation::{Animation, Keyframe, Track};
use crate::board::Board;
use crate::devices::{Actuator, Device};
use crate::errors::Error;
use crate::errors::HardwareError::IncompatibleMode;
use crate::pause;
use crate::protocols::{Pin, PinMode, PinModeId, Protocol};
use crate::utils::{Easing, task};
use crate::utils::scale::Scalable;
use crate::utils::task::TaskHandler;

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
    /// Indicates the current LED intensity when ON.
    intensity: u16,

    // ########################################
    // # Volatile utility data.
    /// If the pin can do PWM, we store that mode here (memoization use only).
    #[cfg_attr(feature = "serde", serde(skip))]
    pwm_mode: Option<PinMode>,
    #[cfg_attr(feature = "serde", serde(skip))]
    protocol: Box<dyn Protocol>,
    /// Inner handler to the task running the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    interval: Arc<Option<TaskHandler>>,
    /// Inner handler to the task running the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    animation: Arc<Option<Animation>>,
}

impl Led {
    pub fn new(board: &Board, pin: u16) -> Result<Self, Error> {
        let mut protocol = board.get_protocol();

        // Get the PWM mode if any
        let pwm_mode = {
            let hardware = protocol.get_hardware().write();
            let _pin = hardware.get_pin(pin)?;
            _pin.supports_mode(PinModeId::PWM)
        };

        // Set pin mode to OUTPUT/PWM accordingly.
        match pwm_mode {
            None => protocol.set_pin_mode(pin, PinModeId::OUTPUT)?,
            Some(_) => protocol.set_pin_mode(pin, PinModeId::PWM)?,
        };

        Ok(Self {
            pin,
            state: Arc::new(RwLock::new(0)),
            default: 0,
            intensity: 0xFF,
            pwm_mode,
            protocol,
            interval: Arc::new(None),
            animation: Arc::new(None),
        })
    }

    /// Turn the LED on.
    pub fn on(&mut self) -> Result<&Self, Error> {
        self.set_state(self.intensity)?;
        Ok(self)
    }

    /// Turn the LED off.
    pub fn off(&mut self) -> Result<&Self, Error> {
        self.set_state(0)?;
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
    /// This is an interval operation and can be stopped by calling [`Led::stop()`].
    pub fn blink(&mut self, ms: u64) -> &Self {
        let mut self_clone = self.clone();

        self.interval = Arc::new(Some(
            task::run(async move {
                loop {
                    self_clone.on()?;
                    pause!(ms);
                    self_clone.off()?;
                    pause!(ms);
                }
                #[allow(unreachable_code)]
                Ok(())
            })
            .unwrap(),
        ));

        self
    }

    // @todo
    // pub async fn pulse(&mut self, ms: u64) -> &Self {}

    /// Stops the current animation. This does not necessarily turn off the LED;
    /// it will remain in its current state when stopped.
    pub fn stop(&self) -> &Self {
        match &self.interval.as_ref() {
            None => {}
            Some(handler) => handler.abort(),
        }
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

    /// Retrieves the LED current intensity in percentage (0-100%).
    pub fn get_intensity(&self) -> u8 {
        // Compute the intensity percentage (depending on resolution (255 on arduino for instance))
        self.intensity.scale(
            0,
            2u16.pow(self.pwm_mode.unwrap().resolution as u32),
            0,
            100,
        )
    }

    /// Set the LED intensity (integer between 0-100) in percent of the max brightness. If a number
    /// higher than 100 is used, the intensity is set to 100%.
    /// If the requested intensity is 100%, the LED will reset to simple on/off (OUTPUT) mode.
    ///
    /// # Parameters
    /// * `intensity`: the requested intensity (between 0-100%)
    ///
    /// # Errors
    /// * `IncompatibleMode`: this function will bail an error if the LED pin does not support PWM.
    pub fn set_intensity(mut self, intensity: u8) -> Result<Self, Error> {
        // Intensity can only be between 0 and 100%
        let mut intensity = intensity.clamp(0, 100) as u16;

        // If the LED can use pwm mode: update the intensity
        self.pwm_mode.ok_or(IncompatibleMode {
            mode: PinModeId::PWM,
            pin: self.pin,
            context: "set LED intensity",
        })?;

        // Compute the intensity value (depending on resolution (255 on arduino for instance))
        intensity = intensity.scale(
            0,
            100,
            0,
            2u16.pow(self.pwm_mode.unwrap().resolution as u32),
        );

        // If the value is higher than the intensity, we update it on the spot.
        if self.state.read().gt(&intensity) {
            self.set_state(intensity)?;
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
            "LED (pin={}) [state={}, default={}, intensity={}]",
            self.pin,
            self.state.read(),
            self.default,
            self.intensity
        )
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Device for Led {}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Actuator for Led {
    fn animate(&mut self, state: u16, duration: u64, transition: Easing) {
        let mut animation = Animation::from(
            Track::new(self.clone())
                .with_keyframe(Keyframe::new(state, 0, duration).set_transition(transition)),
        );
        animation.play();
        self.animation = Arc::new(Some(animation));
    }

    /// Internal only: Update the LED to the target state.
    /// /!\ No checks are made on the state validity.
    fn set_state(&mut self, state: u16) -> Result<u16, Error> {
        match self.get_pin_info()?.mode.id {
            // on/off digital operation.
            PinModeId::OUTPUT => self.protocol.digital_write(self.pin, state > 0),
            // pwm (brightness) mode.
            PinModeId::PWM => self.protocol.analog_write(self.pin, state),
            id => Err(Error::from(IncompatibleMode {
                mode: id,
                pin: self.pin,
                context: "update LED",
            })),
        }?;
        *self.state.write() = state;
        Ok(state)
    }

    /// Retrieves the actuator current state.
    fn get_state(&self) -> u16 {
        self.state.read().clone()
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

// impl Drop for Led {
//     fn drop(&mut self) {
//         let _ = self.set_state(self.get_default());
//     }
// }
