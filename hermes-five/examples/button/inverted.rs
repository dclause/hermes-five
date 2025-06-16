//! Demonstrates the usage of inverted push Button: either pull up or down inverted buttons
//! have their press/release state inverted compared to the real value.

use hermes_five::devices::{Button, InputEvent};
use hermes_five::hardware::{Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    let board = Board::start().unwrap();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let button_inverted = Button::new_inverted_pulldown(&board, 2)?;

        button_inverted.on(InputEvent::OnChange, |value: bool| async move {
            println!("Inverted button value changed: {}", value);
        });
        button_inverted.on(InputEvent::OnPress, |_: bool| async move {
            println!("Inverted button pressed");
        });
        button_inverted.on(InputEvent::OnRelease, |_: bool| async move {
            println!("Inverted button released");
        });

        let pullup_button_inverted = Button::new_inverted_pullup(&board, 8)?;
        pullup_button_inverted.on(InputEvent::OnChange, |value: bool| async move {
            println!("Inverted pullup button value changed: {}", value);
        });
        pullup_button_inverted.on(InputEvent::OnPress, |_: bool| async move {
            println!("Inverted pullup button pressed");
        });
        pullup_button_inverted.on(InputEvent::OnRelease, |_: bool| async move {
            println!("Inverted pullup button released");
        });

        Ok(())
    });
}
