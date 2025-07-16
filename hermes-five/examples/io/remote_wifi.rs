//! This example shows how to use the `RemoteIo` protocol with the `WiFi` transport layer.
//!
//! # Warning
//! Your board must run the proper Firmata WiFi sketch:
//! https://github.com/firmata/arduino/blob/main/examples/StandardFirmataWiFi/StandardFirmataWiFi.ino

use hermes_five::devices::Led;
use hermes_five::hardware::Board;
use hermes_five::transports::WiFi;

#[hermes_five::runtime]
async fn main() {
    // Initialize a TCP connection with the given IP board.
    let board = Board::from(WiFi::new("127.0.0.1:3030")).open().unwrap();

    // Note: Equivalent with full syntax:
    // let board = Board::new(RemoteIo::from(WiFi::new("127.0.0.1:3030"))).connect().unwrap();

    board.on_ready(|board: Board| async move {
        let mut led = Led::new(&board, 2, false)?;
        led.blink(500);
        Ok(())
    });
}
