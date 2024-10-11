use hermes_five::Board;
use hermes_five::devices::Led;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::, |board: Board| async move {
        let mut led = Led::new(&board, 11, false)?;

        // Fade the led to 50% brightness in 500ms.
        led.animate(50, 500, Easing::Linear);

        Ok(())
    });
}
