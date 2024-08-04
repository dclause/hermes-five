use hermes_five::Board;
use hermes_five::devices::Led;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on("ready", |board: Board| async move {
        let mut led = Led::new(&board, 11)?;

        // Fade the led to 50% intensity in 500ms.
        led.animate(50, 500, Easing::Linear);

        Ok(())
    });
}
