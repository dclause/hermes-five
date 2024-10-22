use hermes_five::devices::{Output, Servo};
use hermes_five::{pause, Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a Servo on pin 9.
        let mut servo = Servo::new(&board, 9, 90).expect("Servo is instantiated");

        servo.sweep(500);
        pause!(5000);
        servo.stop();
        servo.reset()?;

        Ok(())
    });
}
