use hermes_five::{Board, BoardEvent, pause};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        println!("Connection done on board.");
        pause!(1000);
        board.close();
        pause!(1000);
        Ok(())
    });

    board.on(BoardEvent::OnClose, |_: Board| async move {
        println!("Connection closed on board.");
        Ok(())
    });
}
