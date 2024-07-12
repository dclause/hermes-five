use hermes_five::Board;

#[hermes_macros::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::run().await;

    board
        .on("ready", |board: Board| async move {
            println!("Board connected: {}", board);
            println!("Pins {:#?}", board.lock().unwrap().pins);
        })
        .await;
}
