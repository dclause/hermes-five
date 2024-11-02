//! You may need a pwm output for an actuator that we have not thought of.
//! This example shows how to use the PwmOutput generic device type to do so.
use hermes_five::devices::PwmOutput;
use hermes_five::hardware::{Board, BoardEvent, Hardware};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        // This is a generic analog device on pin 3.
        let mut output = PwmOutput::new(&board, 3, 0)?;

        // Changes the device to a specific value.
        output.set_value(42)?;

        // Wait for 5secs.
        pause!(5000);

        // Changes the device to a percentage of the possible pin range.
        output.set_percentage(50)?;

        // Disconnect the board since we finished with it.
        board.close();

        Ok(())
    });
}
