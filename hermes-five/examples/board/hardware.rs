use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::io::PinModeId;

#[hermes_five::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::run();

    board.on(BoardEvent::OnReady, |mut board: Board| async move {
        println!("Board connected: {}", board);
        println!("Pins {:#?}", board.get_io().pins);

        // Example using the low-level capability of board to use hardware.
        board.set_pin_mode(13, PinModeId::OUTPUT)?;
        // Turns the pin 13 (embedded led on arduino) to 13.
        board.digital_write(13, true)?;

        Ok(())
    });
}
