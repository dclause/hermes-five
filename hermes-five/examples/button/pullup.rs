//! Demonstrates the simple usage of a push Button on pin 8 configured in PULLUP mode (using the internal resistor) as per show on Arduino tutorial:
//! https://docs.arduino.cc/built-in-examples/digital/InputPullupSerial/

use hermes_five::devices::{Button, InputEvent};
use hermes_five::{Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a PUL-UP Button on pin 8.
        let button = Button::new_pullup(&board, 8)?;

        // Triggered function when the button state changes.
        button.on(InputEvent::OnChange, |value: bool| async move {
            println!("Push button value changed: {}", value);
            Ok(())
        });

        // Triggered function when the button is pressed.
        button.on(InputEvent::OnPress, |_: ()| async move {
            println!("Push button pressed");
            Ok(())
        });

        // Triggered function when the button is released.
        button.on(InputEvent::OnRelease, |_: ()| async move {
            println!("Push button released");
            Ok(())
        });

        Ok(())
    });
}
