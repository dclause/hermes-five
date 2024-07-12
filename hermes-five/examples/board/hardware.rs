use hermes_five::Board;
use hermes_five::protocols::PinModeId;

#[hermes_macros::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::run().await;

    board
        .on("ready", |mut board: Board| async move {
            println!("Board connected: {}", board);
            println!("Pins {:#?}", board.hardware().pins);

            // Example using the low-level capability of board to use hardware.
            board.set_pin_mode(11, PinModeId::OUTPUT).unwrap();
            board.digital_write(11, true).unwrap();
        })
        .await;
}
