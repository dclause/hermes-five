//! Example demonstrating how to use an LED in sink mode.
//!
//! In sink mode, the LED cathode (-) is connected to the board pin configured as a sink (pulling to GND),
//! and the anode (+) is connected to the positive voltage (+5V) through a resistor.
//! To turn the LED on, the board pin is driven LOW (0V), sinking current through the LED.
//! To turn it off, the pin is driven HIGH (+5V).
//!
//! This example uses pin 13 (default Arduino onboard LED).

use hermes_five::devices::Led;
use hermes_five::hardware::Board;
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::start().unwrap();

    board.on_ready(|board: Board| async move {
        // Register a LED on pin 13 (default arduino led): OFF by default.
        // Notice how we simply used `new_sink` instead of `new` here:
        // the rest of the code is the same as usual.
        let mut led = Led::new_sink(&board, 13, false)?;

        // Turn the LED on.
        led.turn_on()?;

        // Wait for 5secs.
        pause!(5000);

        // Turn the LED off.
        led.turn_off()?;

        Ok(())
    });
}
