use crate::board::Board;
use crate::protocols::{Error, IncompatibleMode, Pin, PinMode, PinModeId, Protocol};
use crate::utils::helpers::MapRange;
use crate::utils::task;
use crate::utils::task::TaskHandler;

pub struct Led {
    protocol: Box<dyn Protocol>,
    pin: u16,

    /// Indicates the LED current status.
    is_on: bool, // @todo remove?
    /// Indicates if the LED is running an animation.
    is_running: bool, // @todo remove?
    /// Indicates the current LED value (to be set to the pin)
    value: u16,
    /// Indicates the current LED intensity when ON.
    intensity: u16,
    /// Inner handler to the task running the animation.
    interval: Option<TaskHandler>,
    /// If the pin can do PWM, we store that mode here.
    pwm_mode: Option<PinMode>,
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
            value: 0,
            intensity: 0xFF,
            interval: None,
            pwm_mode,
        })
    }

    /// Set the LED intensity in percent of the max brightness.
    /// Note: this function will bail an error if the LED pin does not support PWM.
    ///
    /// # Parameters
    /// * `intensity`: the requested intensity (between 0-100%)
    pub fn with_intensity(mut self, intensity: u8) -> Result<Self, Error> {
        // Intensity can only be between 0 and 100%
        let intensity = intensity.clamp(0, 100) as u16;

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
                self.intensity = intensity.map(
                    0,
                    100,
                    0,
                    2u16.pow(self.pwm_mode.clone().unwrap().resolution as u32),
                );

                // If the value is higher than the intensity, we update it on the spot.
                if self.value > intensity {
                    self.value = self.intensity;
                    self.update()?;
                }

                Ok(self)
            }
        }
    }

    /// Turn the LED on.
    pub fn on(&mut self) -> Result<&Self, Error> {
        self.is_on = true;
        self.value = self.intensity;
        self.update()
    }

    /// Turn the LED off.
    pub fn off(&mut self) -> Result<&Self, Error> {
        self.is_on = false;
        self.value = 0;
        self.update()
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
    pub async fn blink(&mut self, ms: u64) {
        let mut self_clone = self.clone();

        self.interval = Some(
            task::run(async move {
                loop {
                    self_clone.on()?;
                    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                    self_clone.off()?;
                    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                }
                #[allow(unreachable_code)]
                Ok(())
            })
            .await
            .unwrap(),
        );
    }

    /// Stops the current animation. This does not necessarily turn off the LED;
    /// it will remain in its current state when stopped.
    pub fn stop(&self) {
        match &self.interval {
            None => {}
            Some(handler) => handler.abort(),
        }
    }

    /// Update the LED.
    pub fn update(&mut self) -> Result<&Self, Error> {
        match self.pin()?.mode.id {
            // on/off digital operation.
            PinModeId::OUTPUT => self.protocol.digital_write(self.pin, self.value > 0),
            // pwm (brightness) mode.
            PinModeId::PWM => self.protocol.analog_write(self.pin, self.value),
            id => Err(IncompatibleMode {
                mode: id,
                pin: self.pin,
                operation: String::from("update LED"),
            }),
        }?;
        Ok(self)
    }

    // @todo move this to device
    pub fn pin(&self) -> Result<Pin, Error> {
        let lock = self.protocol.hardware().read();
        Ok(lock.get_pin(self.pin)?.clone())
    }
}

impl Clone for Led {
    fn clone(&self) -> Self {
        Self {
            protocol: self.protocol.clone(),
            pin: self.pin,
            is_on: self.is_on,
            is_running: self.is_running,
            value: self.value,
            intensity: self.intensity,
            interval: None,
            pwm_mode: self.pwm_mode.clone(),
        }
    }
}
