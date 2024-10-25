//! Demonstrates the simple usage an analog sensor: a potentiometer on Arduino pin A0.
//! https://docs.arduino.cc/built-in-examples/analog/AnalogInput/

use hermes_five::devices::{AnalogInput, InputEvent};
use hermes_five::hardware::{Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a Sensor on pin 14 (A0).
        let potentiometer = AnalogInput::new(&board, "A0")?;
        //
        // Triggered function when the button state changes.
        potentiometer.on(InputEvent::OnChange, |value: u16| async move {
            println!("Sensor value changed: {}", value);
            Ok(())
        });

        Ok(())
    });
}
