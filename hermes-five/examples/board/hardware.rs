use hermes_five::hardware::{Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();
    board.on(BoardEvent::OnReady, |mut board: Board| async move {
        println!("Pins {:#?}", board.get_io().pins);
        Ok(())
    });
}
