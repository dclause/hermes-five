use hermes_five::animations::{Animation, Easing, Keyframe, Segment, Track};
use hermes_five::devices::{Led, Servo};
use hermes_five::hardware::Board;

#[hermes_five::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::start().unwrap();

    board.on_ready(|board: Board| async move {
        let servo = Servo::new(&board, 9, 0)?;
        let led = Led::new(&board, 11, false)?;

        let mut animation_servo = Animation::from(
            Segment::from(
                Track::new(servo)
                    .with_keyframe(Keyframe::new(180, 0, 500).set_transition(Easing::SineInOut))
                    .with_keyframe(Keyframe::new(90, 1000, 2000).set_transition(Easing::SineInOut)),
            )
            .set_fps(100)
            .set_repeat(true),
        );

        let mut animation_led = Animation::from(
            Segment::from(
                Track::new(led)
                    .with_keyframe(Keyframe::new(0, 0, 500).set_transition(Easing::SineInOut))
                    .with_keyframe(Keyframe::new(250, 500, 1000).set_transition(Easing::SineInOut)),
            )
            .set_fps(100)
            .set_repeat(true),
        );

        animation_servo.play();
        animation_led.play();

        Ok(())
    });
}
