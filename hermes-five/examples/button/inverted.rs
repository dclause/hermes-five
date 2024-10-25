//! Demonstrates the usage of inverted push Button: either pull up or down inverted buttons
//! have their press/release state inverted compared to the real value.

use hermes_five::devices::{Button, InputEvent};
use hermes_five::hardware::{Board, BoardEvent};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let button_inverted = Button::new_inverted(&board, 2)?;

        button_inverted.on(InputEvent::OnChange, |value: bool| async move {
            println!("Inverted button value changed: {}", value);
            Ok(())
        });
        button_inverted.on(InputEvent::OnPress, |_: ()| async move {
            println!("Inverted button pressed");
            Ok(())
        });
        button_inverted.on(InputEvent::OnRelease, |_: ()| async move {
            println!("Inverted button released");
            Ok(())
        });

        let pullup_button_inverted = Button::new_inverted_pullup(&board, 8)?;
        pullup_button_inverted.on(InputEvent::OnChange, |value: bool| async move {
            println!("Inverted pullup button value changed: {}", value);
            Ok(())
        });
        pullup_button_inverted.on(InputEvent::OnPress, |_: ()| async move {
            println!("Inverted pullup button pressed");
            Ok(())
        });
        pullup_button_inverted.on(InputEvent::OnRelease, |_: ()| async move {
            println!("Inverted pullup button released");
            Ok(())
        });
        Ok(())
    });
}
