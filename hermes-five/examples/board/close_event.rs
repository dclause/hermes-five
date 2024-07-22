use hermes_five::{Board, pause};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board
        .on("ready", |board: Board| async move {
            println!("Connection done on board.");
            pause!(1000);
            board.close();
        })
        .await;

    board
        .on("close", |_: Board| async move {
            println!("Connection closed on board.");
        })
        .await;
}
