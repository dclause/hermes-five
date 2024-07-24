use hermes_five::Board;
use hermes_five::errors::Error;

// NOTE: If you only use sync version of the code, you don't need runtime at all.
// The idea remains the same, excepted no callbacks, events or async code can be used.

fn main() -> Result<(), Error> {
    let board = Board::default().blocking_open()?;
    println!("Board connected: {}", board);
    println!("Pins {:#?}", board.hardware().pins);
    Ok(())
}
