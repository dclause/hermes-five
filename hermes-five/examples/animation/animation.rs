use std::time::SystemTime;

use hermes_five::{Board, pause};
use hermes_five::animation::{Animation, Keyframe, Segment, Track};
use hermes_five::devices::Servo;
use hermes_five::utils::Easing;

#[hermes_five::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::run();

    board
        .on("ready", |board: Board| async move {
            let servo = Servo::new(&board, 9, 0).unwrap();

            let mut animation = Animation::from(
                Segment::from(
                    Track::new(servo)
                        .with_keyframe(Keyframe::new(180, 0, 500).set_transition(Easing::SineInOut))
                        .with_keyframe(
                            Keyframe::new(90, 1000, 2000).set_transition(Easing::SineInOut),
                        ),
                )
                .set_fps(100)
                .set_repeat(true),
            );

            animation.play();
            println!("This will print immediately");
            pause!(3000);
            println!("This will print 3 seconds later");

            animation.stop();
        })
        .await;
}
