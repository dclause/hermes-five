use hermes_five::devices::{Led, Output};
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a LED on pin 13 (default arduino led).
        let mut led = Led::new(&board, 13, false)?;

        // Blinks the LED every 100ms.
        led.blink(100);

        // Notice how blink is not blocker for the current thread, yet it is for the runtime
        println!("This will print immediately");

        // Stops the LED animation.
        pause!(5000);
        led.stop();
        println!("Animation stopped after 5 seconds.");

        Ok(())
    });
}
