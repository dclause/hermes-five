use hermes_five::devices::{Output, Servo};
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;
use hermes_five::utils::Easing;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a Servo on pin 9.
        let mut servo = Servo::new(&board, 9, 0).expect("Servo is instantiated");

        // Animate to end
        servo.animate(180, 500, Easing::SineInOut);
        pause!(500);
        // Animate to start
        servo.animate(0, 500, Easing::SineInOut);
        pause!(500);
        // Animate to default
        servo.animate(servo.get_default(), 500, Easing::SineInOut);
        pause!(500);

        Ok(())
    });
}
