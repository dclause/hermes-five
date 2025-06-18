//! Demonstrates how to use and control INPUT/OUTPUT devices through the PCF8575 expander.

use hermes_five::devices::Led;
use hermes_five::hardware::{Board, BoardEvent, PCF8575};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::start().unwrap();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let pcf8575 = PCF8575::default(&board)?;

        // Register leds for each of the 16 channels of the PCF8575.
        let mut leds = Vec::new();
        for i in 0..16 {
            leds.push(Led::new_sink(&pcf8575, i, false)?);
        }

        // Create a LED chaser.
        loop {
            for curr in 0..16 {
                let prev = if curr == 0 { 15 } else { curr - 1 };
                leds[prev].turn_off()?;
                leds[curr].turn_on()?;
                pause!(100);
            }
        }

        #[allow(unreachable_code)]
        Ok(())
    });
}
