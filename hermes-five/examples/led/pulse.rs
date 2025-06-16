use hermes_five::devices::{Led, OutputDevice};
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::start().unwrap();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a LED on pin 13 (default arduino led).
        let mut led = Led::new(&board, 8, false)?;

        // Pulse the LED every 500ms.
        led.pulse(500);

        // Wait for 5 seconds.
        pause!(5000);

        // stop() stops the animation.
        // off() shuts the led off.
        led.stop();
        led.turn_off()?;

        Ok(())
    });
}
