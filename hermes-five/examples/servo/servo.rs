use hermes_five::{Board, pause};
use hermes_five::devices::Servo;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on("ready", |board: Board| async move {
        // Register a Servo on pin 9.
        let mut servo = Servo::new(&board, 9, 0).expect("Servo is instantiated");

        // Swipe the servo.
        loop {
            servo.to(0)?;
            pause!(1000);
            servo.to(180)?;
            pause!(1000);
        }
    });
}
