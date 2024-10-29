//! This example shows the minimal requirement: a board to control.
//!
//! The board(s) may be defined in various way as they can use different [`IoProtocol`] / [`IoTransport`].
//! A board uses a 'protocol' which defines how to communicate with the software. The default is [`Firmata`] which itself
//! can use various [`IoTransport`] underneath (serial, bluetooth, wifi, etc.).

use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::io::Firmata;
use hermes_five::io::Serial;

#[hermes_five::runtime]
async fn main() {
    // The easiest way to register a board is the `Board::run()` method which both instantiates a Board with all default
    // protocol and transport (firmata+serial) but also immediately opens the communication.
    let board = Board::run();

    // The equivalent would be:
    // Board::default().open();

    // You can customize the protocol used by the board with `Board::new()`. All the following examples are equivalent:
    Board::new(Firmata::default());
    Board::new(Firmata::new("COM3")); // custom port
    Board::new(Firmata::from(Serial::new("COM3"))); // custom transport
    let _ = Board::from(Serial::default()); // Firmata + serial with default port.

    // Beware: the program will stop here since no work as been registered through the `BoardEvent::OnReady` event.
    // Find more about this in the 'examples/board/creation.rs' example.
    board.on(BoardEvent::OnReady, |_: Board| async move {
        // Do something here !
        Ok(())
    });
}
