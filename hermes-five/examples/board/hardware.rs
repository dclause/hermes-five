//! This example shows how to access and control the hardware associated with a board: low level style!

use hermes_five::hardware::{Board, BoardEvent, LowLevelApi, PinModeId};

#[hermes_five::runtime]
async fn main() {
    let board = Board::start().unwrap();

    board.on(BoardEvent::OnReady, |board: Board| async move {
        println!("Protocol {:#?}", board.get_protocol_name());
        println!(
            "Firmware {:#?} version={}",
            board.get_firmware_name(),
            board.get_firmware_version()
        );
        println!("Pins {:#?}", board.get_pins());

        // Set the embedded led ON.
        // This is shown for showcase but using a device (see led examples) is recommended.
        board.set_pin_mode(13, PinModeId::OUTPUT)?;
        board.digital_write(13, true)?;

        Ok(())
    });
}
