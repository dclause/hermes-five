use hermes_five::animation::{Animation, Keyframe, Track};
use hermes_five::Board;
use hermes_five::devices::Servo;

#[hermes_five::runtime]
async fn main() {
    // Default board: uses SerialProtocol communication via the first available port.
    let board = Board::run().await;

    board
        .on("ready", |board: Board| async move {
            let servo = Servo::new(&board, 9).unwrap();

            Animation::from(
                Track::new(servo)
                    .with_keyframe(Keyframe::new(0, 2000))
                    .with_keyframe(Keyframe::new(180, 2000)),
            )
            .set_repeat(true)
            .play()
            .await;
        })
        .await;
}
