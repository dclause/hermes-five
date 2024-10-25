use hermes_five::hardware::{Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        println!("Board connected: {}", board);
        println!("Pins {:#?}", board.get_io().pins);

        Ok(())
    });
}
