use hermes_five::devices::{Output, Servo};
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a Servo on pin 9.
        let mut servo = Servo::new(&board, 9, 90).expect("Servo is instantiated");

        // Move to end
        servo.to(180)?;
        pause!(500);
        // Move to start
        servo.to(0)?;
        pause!(500);
        // Move to default
        servo.reset()?;
        pause!(500);

        Ok(())
    });
}
