//! Demonstrates the simple usage a digital sensor: a microwave sensor on Arduino pin 7
//! Example with DFRobot SEN0192: https://wiki.dfrobot.com/MicroWave_Sensor_SKU__SEN0192

use hermes_five::devices::{DigitalInput, InputEvent};
use hermes_five::hardware::{Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a sensor on pin 7.
        let sensor = DigitalInput::new(&board, 7)?;
        //
        // Triggered function when the button state changes.
        sensor.on(InputEvent::OnChange, |value: bool| async move {
            println!("Sensor value changed: {}", value);
            Ok(())
        });

        Ok(())
    });
}
