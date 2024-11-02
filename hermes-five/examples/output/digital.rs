//! You may need an ON/OFF actuator that we have not thought of, or that does not really need its own implementation (like a LED, sensor, etc.)
//! This example shows how to use the DigitalOutput generic device type to do so.
use hermes_five::devices::DigitalOutput;
use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // This is a generic ON/OFF device on pin 13.
        let mut output = DigitalOutput::new(&board, 13, false)?;

        // Turn the device on.
        output.turn_on()?;

        // Wait for 5secs.
        pause!(5000);

        // Turn the device off.
        output.turn_off()?;

        // Disconnect the board since we finished with it.
        board.close();

        Ok(())
    });
}
