use hermes_five::Board;

#[hermes_five::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::run();

    board.on("ready", |board: Board| async move {
        println!("Board connected: {}", board);
        println!("Pins {:#?}", board.hardware().pins);

        Ok(())
    });
}
