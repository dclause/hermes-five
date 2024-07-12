use crate::board::Board;
use crate::protocols::{
    Error, IncompatiblePin, IncompatibleValue, Pin, PinMode, PinValue, UnknownPin,
};

pub struct Led {
    // @todo board and pin should be in a Device trait ?
    board: Board,
    pin: Pin,

    is_on: bool,
    is_running: bool,
    value: PinValue,
    intensity: PinValue,
    interval: u8,
}

impl Led {
    pub fn new(board: Board, pin: usize) -> Result<Self, Error> {
        let board_pin: Pin = match board.pins.iter().find(|&p| p.id == pin as u8) {
            None => Err(UnknownPin { pin: pin as u8 }),
            Some(p) => Ok(p.clone()),
        }?;
        Ok(Self {
            board,
            pin: board_pin,
            is_on: false,
            is_running: false,
            value: PinValue::LOW,
            intensity: PinValue::MAX_PWM,
            interval: 0,
        })
    }

    /// Set the LED intensity.
    /// Note: this function will bail an error if the LED pin does not support PWM.
    ///
    /// # Parameters
    /// * `intensity`: the requested intensity
    pub fn with_intensity(&mut self, intensity: PinValue) -> Result<&Self, Error> {
        if !self.pin.supported_modes.contains(&PinMode::PWM) {
            return Err(IncompatiblePin { pin: self.pin.id });
        }
        self.intensity = intensity;
        Ok(self)
    }

    /// Turn the LED on.
    pub fn on(&mut self) -> Result<&Self, Error> {
        self.is_on = true;
        self.value = PinValue::HIGH;
        self.update()
    }

    /// Turn the LED off.
    pub fn off(&mut self) -> Result<&Self, Error> {
        self.is_on = false;
        self.value = PinValue::LOW;
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

    pub fn update(&mut self) -> Result<&Self, Error> {
        // Send the value appropriately
        match self.value == PinValue::LOW || self.value == PinValue::HIGH {
            // value is digital:
            true => match self.pin.supported_modes.contains(&PinMode::OUTPUT) {
                // Send the value as digital
                true => self
                    .board
                    .digital_write(self.pin.id as i32, self.value as i32),
                // Bail an error if the pin is OUTPUT incompatible.
                false => Err(IncompatibleValue {
                    value: self.value as u8,
                }),
            },
            // value is analog:
            false => match self.pin.supported_modes.contains(&PinMode::PWM) {
                true => {
                    let channel = self
                        .pin
                        .channel
                        .ok_or(IncompatiblePin { pin: self.pin.id })?;
                    self.board.analog_write(channel as i32, self.value as i32)
                }
                // Bail an error if the pin is PWM incompatible.
                false => Err(IncompatibleValue {
                    value: self.value as u8,
                }),
            },
        }?;
        Ok(self)
    }
}
