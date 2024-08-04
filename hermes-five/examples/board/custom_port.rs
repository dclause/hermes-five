use hermes_five::Board;
use hermes_five::protocols::SerialProtocol;

#[hermes_five::runtime]
async fn main() {
    // Notice how you have to explicitly `open()` the board connection when using the builder.
    let board = Board::build(SerialProtocol::new("COM4")).open();

    board.on("ready", |_: Board| async move {
        // board.pinMode(13, board.MODES.OUTPUT);
        //
        for _ in 0..500 {
            // Whatever the last value was, write the opposite
            // board.digitalWrite(13, board.pins[13].value ? 0 : 1);
        }

        Ok(())
    });
}
