use async_trait::async_trait;

use crate::board::Board;
use crate::devices::{Actuator, Device};
use crate::errors::{Error, IncompatibleMode};
use crate::pause;
use crate::protocols::{Pin, PinMode, PinModeId, Protocol};
use crate::utils::Easing;
use crate::utils::scale::Scalable;
use crate::utils::task;
use crate::utils::task::TaskHandler;

pub struct Led {
    protocol: Box<dyn Protocol>,
    pin: u16,

    /// Indicates the LED current status.
    is_on: bool, // @todo remove?
    /// Indicates if the LED is running an animation.
    is_running: bool, // @todo remove?
    /// Indicates the current LED intensity when ON.
    intensity: u16,
    /// If the pin can do PWM, we store that mode here.
    pwm_mode: Option<PinMode>,

    // # Actuator
    /// Indicates the current LED state
    state: u16,
    /// Inner handler to the task running the animation.
    interval: Option<TaskHandler>,
}

impl Led {
    pub fn new(board: &Board, pin: u16) -> Result<Self, Error> {
        let pwm_mode;

        // Set pin mode to OUTPUT
        let mut protocol = board.protocol();
        protocol.set_pin_mode(pin, PinModeId::OUTPUT)?;

        // Get the PWM mode if any
        {
            let hardware = protocol.hardware().write();
            let _pin = hardware.get_pin(pin)?;
            pwm_mode = _pin.get_mode(PinModeId::PWM);
        }

        Ok(Self {
            protocol: board.protocol(),
            pin,
            is_on: false,
            is_running: false,
            state: 0,
            intensity: 0xFF,
            interval: None,
            pwm_mode,
        })
    }

    /// Set the LED intensity (integer between 0-100) in percent of the max brightness. If a number
    /// higher than 100 is used, the intensity is set to 100%.
    /// If the requested intensity is 100%, the led will reset to simple on/off (OUTPUT) mode.
    ///
    /// # Parameters
    /// * `intensity`: the requested intensity (between 0-100%)
    ///
    /// # Errors
    /// * `IncompatibleMode`: this function will bail an error if the LED pin does not support PWM.
    pub fn with_intensity(mut self, intensity: u8) -> Result<Self, Error> {
        // Intensity can only be between 0 and 100%
        let mut intensity = intensity.clamp(0, 100) as u16;

        // If the requested intensity is 100%, let's get back to OUTPUT mode.
        if intensity >= 100 {
            self.intensity = 100;
            self.protocol.set_pin_mode(self.pin, PinModeId::OUTPUT)?;
            return Ok(self);
        }

        // If the LED can use pwm mode: update the intensity
        match self.pwm_mode {
            None => Err(IncompatibleMode {
                mode: PinModeId::PWM,
                pin: self.pin,
                operation: String::from("set LED intensity"),
            }),
            Some(_) => {
                self.protocol.set_pin_mode(self.pin, PinModeId::PWM)?;

                // Compute the intensity value (depending on resolution (255 on arduino for instance))
                intensity = intensity.scale(
                    0,
                    100,
                    0,
                    2u16.pow(self.pwm_mode.unwrap().resolution as u32),
                );

                // If the value is higher than the intensity, we update it on the spot.
                if self.state > intensity {
                    self.set_state(intensity)?;
                }

                Ok(self)
            }
        }
    }

    /// Turn the LED on.
    pub fn on(&mut self) -> Result<&Self, Error> {
        self.is_on = true;
        self.set_state(self.intensity)?;
        Ok(self)
    }

    /// Turn the LED off.
    pub fn off(&mut self) -> Result<&Self, Error> {
        self.is_on = false;
        self.set_state(0)?;
        Ok(self)
    }

    /// Toggle the current state, if on then turn off, if off then turn on.
    pub fn toggle(&mut self) -> Result<&Self, Error> {
        match self.is_on {
            true => self.off(),
            false => self.on(),
        }
    }

    /// Blink the LED on/off in phases of ms (milliseconds) duration.
    /// This is an interval operation and can be stopped by calling [`Led::stop()`].
    pub async fn blink(&mut self, ms: u64) -> &Self {
        let mut self_clone = self.clone();

        self.interval = Some(
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
            .await
            .unwrap(),
        );

        self
    }

    // pub async fn pulse(&mut self, ms: u64) -> &Self {}

    /// Stops the current animation. This does not necessarily turn off the LED;
    /// it will remain in its current state when stopped.
    pub fn stop(&self) -> &Self {
        match &self.interval {
            None => {}
            Some(handler) => handler.abort(),
        }
        self
    }

    // @todo move this to device ?
    pub fn pin(&self) -> Result<Pin, Error> {
        let lock = self.protocol.hardware().read();
        Ok(lock.get_pin(self.pin)?.clone())
    }
}

// @todo make derive
impl Device for Led {}

#[async_trait]
impl Actuator for Led {
    /// Update the LED to the target state.
    /// /!\ No checks are made on the state validity.
    fn set_state(&mut self, state: u16) -> Result<(), Error> {
        self.state = state;
        match self.pin()?.mode.id {
            // on/off digital operation.
            PinModeId::OUTPUT => self.protocol.digital_write(self.pin, self.state > 0),
            // pwm (brightness) mode.
            PinModeId::PWM => self.protocol.analog_write(self.pin, self.state),
            id => Err(IncompatibleMode {
                mode: id,
                pin: self.pin,
                operation: String::from("update LED"),
            }),
        }?;
        Ok(())
    }

    async fn animate(&mut self, target: u16, duration: u32, easing: Easing) {
        let mut self_clone = self.clone();
        let animation_start_value = self_clone.state;
        let animation_end_value = target;

        self.interval = Some(
            task::run(async move {
                let fps = 40f32;
                let tick_ms = (1000f32 / fps) as u32;
                let mut t_ms = 0u32;

                while t_ms < duration {
                    // Current time between (0 - 1).
                    let normalized_t = (t_ms as f32).scale(0f32, duration as f32, 0f32, 1f32);
                    // Current value between (0 - 1)
                    self_clone.set_state(easing.call(normalized_t).scale(
                        0f32,
                        1f32,
                        animation_start_value as f32,
                        animation_end_value as f32,
                    ) as u16)?;
                    t_ms = t_ms + tick_ms;
                    pause!(tick_ms);
                }

                Ok(())
            })
            .await
            .unwrap(),
        );
    }

    // fn denormalize(&mut self, value: f32) {
    //     self.state = match self.pwm_mode {
    //         None => u16::from(value >= 0.5),
    //         Some(mode) => (value * mode.resolution as f32) as u16,
    //     };
    // }
}

impl Clone for Led {
    fn clone(&self) -> Self {
        Self {
            protocol: self.protocol.clone(),
            pin: self.pin,
            is_on: self.is_on,
            is_running: self.is_running,
            state: self.state,
            intensity: self.intensity,
            interval: None,
            pwm_mode: self.pwm_mode.clone(),
        }
    }
}
