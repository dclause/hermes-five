use crate::board::Board;
use crate::protocols::{Error, IncompatibleMode, Pin, PinModeId, UnknownPin};
use crate::utils::helpers::MapRange;

pub struct Led {
    // @todo board and pin should be in a Device trait ?
    board: Board,
    pin: u16,

    is_on: bool,
    is_running: bool,
    value: u16,
    intensity: u16,
    interval: u8,
}

impl Led {
    pub fn new(mut board: Board, pin: u16) -> Result<Self, Error> {
        {
            let board_pin: &mut Pin = board.pins.get_mut(pin as usize).ok_or(UnknownPin { pin })?;
            board_pin.mode = board_pin.get_mode(PinModeId::OUTPUT)?;
        }

        Ok(Self {
            board,
            pin,
            is_on: false,
            is_running: false,
            value: 0,
            intensity: 0xFF,
            interval: 0,
        })
    }

    /// Set the LED intensity in percent of the max brightness.
    /// Note: this function will bail an error if the LED pin does not support PWM.
    ///
    /// # Parameters
    /// * `intensity`: the requested intensity (between 0-100%)
    pub fn with_intensity(&mut self, intensity: u8) -> Result<&Self, Error> {
        // Intensity can only be between 0 and 100%
        let intensity = intensity.clamp(0, 100) as u16;

        // Set the pin as PWM mode if not yet done.
        let pin = self.board.get_pin_mut(self.pin)?;
        let pwm_mode = pin.get_mode(PinModeId::PWM)?;
        pin.mode = pwm_mode.clone();

        // Now set the pin mode in the board.
        let pin_id = pin.id;
        let pwm_mode_id = pwm_mode.id;
        self.board.set_pin_mode(pin_id, pwm_mode_id)?;

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

    /// Blink the LED on/off in phases of ms (milliseconds) duration.
    /// This is an interval operation and can be stopped by calling [`Led::stop()`].
    pub fn blink(&mut self, ms: usize) -> Result<&Self, Error> {
        // @todo implement stop()
        Ok(self)
    }

    /// Stops the current animation. This does not necessarily turn off the LED;
    /// it will remain in its current state when stopped.
    pub fn stop(&self) {
        // @todo implement stop()
    }

    /// Update the LED.
    pub fn update(&mut self) -> Result<&Self, Error> {
        let pin = self.board.get_pin_mut(self.pin)?;

        let pin_id = pin.id;
        match pin.mode.id {
            // on/off digital operation.
            PinModeId::OUTPUT => self.board.digital_write(pin_id, self.value > 0),
            // pwm (brightness) mode.
            PinModeId::PWM => self.board.analog_write(pin_id, self.value),
            _ => Err(IncompatibleMode {
                mode: pin.mode.id,
                pin: pin.id,
                operation: String::from("update LED"),
            }),
        }?;
        Ok(self)
    }
}
