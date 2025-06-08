/// This example shows how to use the `RemoteIo` protocol with the `Serial` transport layer.
///
/// /!\ Your board needs to use properly configured schema:
///`https://github.com/firmata/arduino/blob/main/examples/StandardFirmata/StandardFirmata.ino`

use hermes_five::devices::{Led, Output};
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::io::{RemoteIo, Serial};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {

    // Initialize a serial connection with auto-detected port.
    let board = Board::from(Serial::default()).open();

    // Initialize a serial connection with custom port.
    let board = Board::from(Serial::new("/dev/ttyUSB0")).open();

    // Equivalent with full syntax:
    let board = Board::new(RemoteIo::from(Serial::new("/dev/ttyUSB0"))).open();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        let mut led = Led::new(&board, 13, false)?;
        led.blink(500);
        pause!(5000);
        led.stop();
        Ok(())
    });
}
