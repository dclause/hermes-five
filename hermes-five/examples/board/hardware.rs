use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::io::IO;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();
    board.on(BoardEvent::OnReady, |board: Board| async move {
        println!("Pins {:#?}", board.get_io().read().pins);
        Ok(())
    });
}
