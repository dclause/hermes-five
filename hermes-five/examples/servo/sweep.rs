//! This example demonstrates how to loop sweep a servo in a given range of motion.

use hermes_five::devices::{Output, Servo};
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a Servo on pin 9.
        let mut servo = Servo::new(&board, 9, 90)?;

        // Restricts the servo range of motion.
        let mut servo = servo.set_range([30, 150]);

        // Sweep the servo continuously within the range of motion.
        servo.sweep(500);

        // Wait 5sec.
        pause!(5000);

        // Stops the servo animation.
        servo.stop();

        // Resets the servo to default position.
        servo.reset()?;

        Ok(())
    });
}
