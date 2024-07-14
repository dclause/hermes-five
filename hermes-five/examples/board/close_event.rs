use hermes_five::Board;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run().await;

    board
        .on("ready", |board: Board| async move {
            println!("Connection done on board.");
            hermes_five::utils::sleep(std::time::Duration::from_secs(1)).await;
            board.close().await;
        })
        .await;

    board
        .on("close", |_: Board| async move {
            println!("Connection closed on board.");
        })
        .await;
}
