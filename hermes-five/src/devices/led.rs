use crate::board::Board;
use crate::protocols::{Error, IncompatibleMode, Pin, PinModeId, Protocol, UnknownPin};
use crate::utils::helpers::MapRange;
use crate::utils::task;
use crate::utils::task::TaskHandler;

pub struct Led {
    protocol: Box<dyn Protocol>,
    pin: u16,

    is_on: bool,
    is_running: bool,
    value: u16,
    intensity: u16,
    interval: Option<TaskHandler>,
}

impl Led {
    pub fn new(board: &Board, pin: u16) -> Result<Self, Error> {
        // Force pin mode to OUTPUT
        {
            let protocol = board.protocol();
            let mut hardware = protocol.hardware().write();
            let board_pin = hardware
                .pins
                .get_mut(pin as usize)
                .ok_or(UnknownPin { pin })?;
            board_pin.mode = board_pin.get_plausible_mode(PinModeId::OUTPUT)?;
        }

        Ok(Self {
            protocol: board.protocol(),
            pin,
            is_on: false,
            is_running: false,
            value: 0,
            intensity: 0xFF,
            interval: None,
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
        let pin_id;
        let pwm_mode;

        // Set the pin as PWM mode if possible (bail error otherwise).
        {
            // Lock the protocol hardware for modification.
            let mut lock = self.protocol.hardware().write();
            let pin = lock.get_pin_mut(self.pin)?;
            let mode = pin.get_plausible_mode(PinModeId::PWM)?;
            pin.mode = mode.clone();
            // Now set the pin mode in the board.
            pin_id = pin.id;
            pwm_mode = mode.clone();
        }

        self.protocol.set_pin_mode(pin_id, pwm_mode.id)?;

        // Compute the max intensity value (depending on resolution (255 on arduino for instance))
        self.intensity = intensity.map(0, 100, 0, 2u16.pow(pwm_mode.resolution as u32));

        // If the value is higher than the intensity, we update it on the spot.
        if self.value > intensity {
            self.value = self.intensity;
            self.update()?;
        }

        Ok(self)
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

    // /// Blink the LED on/off in phases of ms (milliseconds) duration.
    // /// This is an interval operation and can be stopped by calling [`Led::stop()`].
    // pub fn blink(&mut self, ms: u64) {
    //     let self_arc = Arc::new(Mutex::new(self.clone()));
    //
    //     let self_clone = Arc::clone(&self_arc);
    //     self.interval = Some(tokio::spawn(async move {
    //         loop {
    //             self_clone.lock().on()?;
    //             tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    //             self_clone.lock().off()?;
    //             tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    //         }
    //         #[allow(unreachable_code)]
    //         Ok(())
    //     }));
    // }

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
        let pin;

        {
            let mut lock = self.protocol.hardware().write();
            pin = lock.get_pin_mut(self.pin)?.clone();
        }

        match pin.mode.id {
            // on/off digital operation.
            PinModeId::OUTPUT => self.protocol.digital_write(pin.id, self.value > 0),
            // pwm (brightness) mode.
            PinModeId::PWM => self.protocol.analog_write(pin.id, self.value),
            _ => Err(IncompatibleMode {
                mode: pin.mode.id,
                pin: pin.id,
                operation: String::from("update LED"),
            }),
        }?;
        Ok(self)
    }

    // @todo move this to device
    pub fn pin(&self) -> Pin {
        let lock = self.protocol.hardware().read();
        lock.get_pin(self.pin).unwrap().clone()
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
        }
    }
}
