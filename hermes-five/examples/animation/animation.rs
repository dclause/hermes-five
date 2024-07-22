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

            // let mut animation = Animation::from(
            //     Track::new(servo)
            //         .with_keyframe(Keyframe::new(1, 100, 30))
            //         .with_keyframe(Keyframe::new(2, 120, 90))
            //         .with_keyframe(Keyframe::new(3, 220, 10))
            //         .with_keyframe(Keyframe::new(4, 200, 110))
            //         .with_keyframe(Keyframe::new(5, 110, 190)),
            // );
            //
            // animation.play().await;

            // let mut animation = Animation::from(
            //     Segment::from(
            //         Track::new(servo)
            //             .with_keyframe(Keyframe::new(0, 0, 1000))
            //             .with_keyframe(Keyframe::new(180, 1000, 2000))
            //             .with_keyframe(Keyframe::new(0, 2000, 2000)),
            //     )
            //     .set_loopback(1000)
            //     .set_repeat(true),
            // );

            let mut animation = Animation::from(
                Segment::from(
                    Track::new(servo)
                        .with_keyframe(
                            Keyframe::new(180, 0, 1000).set_transition(Easing::SineInOut),
                        )
                        .with_keyframe(
                            Keyframe::new(0, 1000, 2000).set_transition(Easing::SineInOut),
                        ),
                )
                .set_fps(60)
                .set_repeat(true),
            );

            animation.play().await;
            println!("This will print immediately");
            pause!(500);
            println!("This will print 3 seconds later");

            animation.stop();
        })
        .await;
}
