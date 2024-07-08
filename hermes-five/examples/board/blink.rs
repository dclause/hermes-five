use hermes_five::entities::Board;

#[hermes_macros::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::run().await;

    board
        .on("ready", |_: Board| async move {
            // Do blink
        })
        .await;
}
