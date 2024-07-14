use hermes_five::{Board, pause};
use hermes_five::devices::Servo;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run().await;

    board
        .on("ready", |board: Board| async move {
            // Register a Servo on pin 9.
            let mut servo = Servo::new(&board, 9).expect("Servo is instantiated");
            println!("{:?}", servo.pin());

            // Swipe the servo.
            loop {
                servo.to(0).unwrap();
                pause!(1000);
                servo.to(180).unwrap();
                pause!(1000);
            }
        })
        .await;
}
