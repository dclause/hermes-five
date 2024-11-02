//! This example shows how to register event handlers on the board.
//! The event handlers are functions that are called when a specific event occurs on the board.
//!
//! In this example, we register two event handlers:
//! - `OnReady`: This event is triggered when the board is ready to use.
//! - `OnClose`: This event is triggered when the board is closed.
//!
//! # Notes
//! - The [`Board.on`] method is used to register the event handlers on the board.
//! - The [`BoardEvent`] structure lists all events a board may emit.
//! - You can register multiple callbacks for a same event.
//! - Callbacks are asynchronous futures, hence the `async move` syntax.
//! - Callbacks follows a strict syntax and MUST be used with the proper argument signature.
//! - All callbacks MUST return `Result<(), Error>>`.

use hermes_five::hardware::{Board, BoardEvent, Hardware};

#[hermes_five::runtime]
async fn main() {
    // This line creates a new board using the default configuration.
    // For more advanced scenarios, consult the 'examples/board/creation.rs' example.
    // Note: this line is equivalent to: `Board::default().open()`
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        println!("Connection done on board.");
        board.close();
        Ok(())
    });

    board.on(BoardEvent::OnClose, |_: Board| async move {
        println!("Connection closed on board.");
        Ok(())
    });

    // Note that you can register as many event handlers as you want on a same event.
    board.on(BoardEvent::OnReady, |_: Board| async move {
        println!("Hello from another event handler!");
        Ok(())
    });
}
