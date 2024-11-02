//! Demonstrates how to use and control a servo through a PWM-driver board like the PCA9685.
//! <https://learn.adafruit.com/16-channel-pwm-servo-driver>

use hermes_five::devices::Servo;
use hermes_five::hardware::{Board, BoardEvent, PCA9685};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let pca9685 = PCA9685::default(&board)?;

        // Register a Servo on channel 0 of the PCA9685.
        let mut servo = Servo::new(&pca9685, 0, 90)?;

        servo.sweep(1000);
        Ok(())
    });
}
