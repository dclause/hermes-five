//! Demonstrates the simple usage of a push Button on pin 2 as per show on Arduino tutorial:
//! https://docs.arduino.cc/built-in-examples/digital/Button/

use hermes_five::devices::{Button, InputEvent};
use hermes_five::hardware::{Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a Button on pin 2.
        let button = Button::new(&board, 2)?;

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
