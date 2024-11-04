use hermes_five::animations::Easing;
use hermes_five::devices::{Led, Output};
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let mut led = Led::new(&board, 13, false)?;

        // Fade the LED to 50% brightness in 1000ms.
        led.animate(0x80, 1000, Easing::Linear);

        pause!(1000);

        // Dim the LED to 0% brightness in 1000ms.
        led.animate(0, 1000, Easing::Linear);

        Ok(())
    });
}
