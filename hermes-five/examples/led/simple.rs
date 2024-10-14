use hermes_five::{Board, BoardEvent, pause};
use hermes_five::devices::Led;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a LED on pin 13 (default arduino led).
        let mut led = Led::new(&board, 13, false)?;

        // Turn the LED on.
        led.on()?;

        // Wait for 5secs.
        pause!(5000);

        // Turn the LED offt.
        led.off()?;

        Ok(())
    });
}
