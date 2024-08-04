use hermes_five::{Board, pause};
use hermes_five::devices::Servo;
use hermes_five::utils::Easing;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on("ready", |board: Board| async move {
        // Register a Servo on pin 9.
        let mut servo = Servo::new(&board, 9, 0).expect("Servo is instantiated");
        println!("{:?}", servo.pin());
        servo.to(180);
        pause!(1000);
        let duration = 2000;
        // Swipe the servo.
        loop {
            servo.animate(0, duration, Easing::SineInOut);
            servo.on("complete", |servo: Servo| async move {
                servo.animate_sync(180, duration, Easing::SineInOut);
            });
            pause!(4000)
        }
    });
}
