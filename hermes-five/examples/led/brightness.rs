use hermes_five::devices::Led;
use hermes_five::hardware::{Board, BoardEvent};

// /!\ Use of brightness requires a PWM compatible pin.
// Consult your board schematics to know which ones are compatible.

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a LED on pin 11.
        let mut led = Led::new(&board, 11, false)?
            // Lower brightness to 50%: this will now impose a PWM compatible pin.
            .set_brightness(50)?;

        led.blink(500);

        Ok(())
    });
}
