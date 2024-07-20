use std::time::SystemTime;

use hermes_five::{Board, pause};
use hermes_five::devices::{Actuator, Servo};
use hermes_five::utils::Easing;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run().await;

    board
        .on("ready", |board: Board| async move {
            // Register a Servo on pin 9.
            let mut servo = Servo::new(&board, 9).expect("Servo is instantiated");
            println!("{:?}", servo.pin());
            servo.to(180);
            pause!(1000);
            let duration = 2000;
            // Swipe the servo.
            loop {
                servo.animate(0, duration, Easing::SineInOut).await;
                servo.on("complete", |servo: Servo| async move {
                    let start = SystemTime::now();
                    servo.animate_sync(180, duration, Easing::SineInOut);
                    let end = SystemTime::now();
                    let elapsed = end.duration_since(start).unwrap().as_millis();
                    println!("Sync animate duration: {}", elapsed);
                });
                pause!(4000)
            }
        })
        .await;
}
