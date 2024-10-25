use hermes_five::animations::{Animation, Keyframe, Segment, Track};
use hermes_five::devices::Servo;
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;
use hermes_five::utils::Easing;

#[hermes_five::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let servo = Servo::new(&board, 22, 0).unwrap();

        // This is the full animation declaration:
        // - an `Animation` contains:
        //      - `Segment`s (added by `with_segment`) each containing:
        //          - `Track`s (added by `with_track`) each containing:
        //              - `Keyframe`s (added by `with_keyframe`)
        // @note: animations can be defined less verbosely using `Animation::from()` shortcuts.
        let mut animation = Animation::default().with_segment(
            Segment::default()
                .with_track(
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

        animation.play();

        Ok(())
    });
}
