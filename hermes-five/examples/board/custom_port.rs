use hermes_five::protocols::SerialProtocol;
use hermes_five::{Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    // Notice how you have to explicitly `open()` the board connection when using the builder.
    let board = Board::from(SerialProtocol::new("COM4")).open();

    board.on(BoardEvent::OnReady, |_: Board| async move {
        // board.pinMode(13, board.MODES.OUTPUT);
        //
        for _ in 0..500 {
            // Whatever the last value was, write the opposite
            // board.digitalWrite(13, board.pins[13].value ? 0 : 1);
        }

        Ok(())
    });
}
