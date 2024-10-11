use hermes_five::{Board, BoardEvent, pause};
use hermes_five::devices::{Actuator, Led};
use hermes_five::utils::Easing;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let mut led = Led::new(&board, 8, false)?;

        // Fade the LED to 50% brightness in 500ms.
        led.animate(0x80u16, 500, Easing::Linear);

        pause!(1000);

        // Dim the LED to 0% brightness in 500ms.
        led.animate(0u16, 500, Easing::Linear);

        Ok(())
    });
}
