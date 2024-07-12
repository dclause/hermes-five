use anyhow::Result;

use hermes_five::Board;

// NOTE: If you only use sync version of the code, you don't need runtime at all.
// The idea remains the same, excepted no callbacks, events or async code can be used.

fn main() -> Result<()> {
    let board = Board::default().blocking_open()?;
    println!("Board connected: {}", board);
    println!("Pins {:#?}", board.hardware().pins);
    Ok(())
}
