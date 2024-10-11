use hermes_five::{Board, BoardEvent};
use hermes_five::devices::Led;

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
