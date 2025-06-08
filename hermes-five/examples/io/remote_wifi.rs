/// This example shows how to use the `RemoteIo` protocol with the `WiFi` transport layer.
///
/// /!\ Your board needs to use properly configured schema:
///`https://github.com/firmata/arduino/blob/main/examples/StandardFirmataWiFi/StandardFirmataWiFi.ino`

use hermes_five::devices::{Led, Output};
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::io::{RemoteIo, WiFi};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {

    // Initialize a TCP connection with the given IP board.
    let board = Board::from(WiFi::new("127.0.0.1:3030")).open();

    // Equivalent with full syntax:
    let board = Board::new(RemoteIo::from(WiFi::new("127.0.0.1:3030"))).open();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let mut led = Led::new(&board, 2, false)?;
        led.blink(500);
        pause!(5000);
        led.stop();
        Ok(())
    });
}
