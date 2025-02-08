//! Demonstrates how to use and control a servo through a PWM-driver board like the PCA9685.
//! <https://learn.adafruit.com/16-channel-pwm-servo-driver>

use hermes_five::devices::{Output, Servo};
use hermes_five::hardware::{Board, BoardEvent, PCA9685};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let pca9685 = PCA9685::default(&board)?;

        // Register servos on channel 0, 1 and 2 of the PCA9685.
        let mut servo0 = Servo::new(&pca9685, 0, 90)?;
        let mut servo1 = Servo::new(&pca9685, 1, 90)?;
        let mut servo2 = Servo::new(&pca9685, 2, 90)?;

        servo0.sweep(1000);
        servo1.sweep(500);
        servo2.sweep(2000);

        pause!(5000);
        servo0.reset()?;
        servo1.reset()?;
        servo2.reset()?;

        Ok(())
    });
}
