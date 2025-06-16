//! This example shows how to access and control the hardware associated with a board: low level style!

use hermes_five::hardware::{Board, BoardEvent, Hardware};
use hermes_five::io::PinModeId::OUTPUT;
use hermes_five::io::IO;

#[hermes_five::runtime]
async fn main() {
    let board = Board::start().unwrap();

    board.on(BoardEvent::OnReady, |mut board: Board| async move {
        println!("Protocol {:#?}", board.get_protocol().get_name());
        println!(
            "Firmware {:#?} version={}",
            board.get_io().read().firmware_name,
            board.get_io().read().firmware_version
        );
        println!("Pins {:#?}", board.get_io().read().pins);

        // Set the embedded led ON.
        // This is shown for showcase but using a device (see led examples) is recommended.
        board.set_pin_mode(13, OUTPUT)?;
        board.digital_write(13, true)?;

        Ok(())
    });
}
