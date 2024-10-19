//! Demonstrates the simple usage of a push Button on pin 2 as per show on Arduino tutorial:
//! https://docs.arduino.cc/built-in-examples/digital/Button/

use hermes_five::{Board, BoardEvent};
use hermes_five::devices::{Button, ButtonEvent};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // Register a Button on pin 2.
        let button = Button::new(&board, 2)?;

        // Triggered function when the button state changes.
        button.on(ButtonEvent::OnChange, |value: bool| async move {
            println!("Push button value changed: {}", value);
            Ok(())
        });

        // Triggered function when the button is pressed.
        button.on(ButtonEvent::OnPress, |_: ()| async move {
            println!("Push button pressed");
            Ok(())
        });

        // Triggered function when the button is released.
        button.on(ButtonEvent::OnRelease, |_: ()| async move {
            println!("Push button released");
            Ok(())
        });

        Ok(())
    });
}
