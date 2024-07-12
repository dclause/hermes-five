use hermes_five::Board;
use hermes_five::devices::Led;

#[hermes_macros::runtime]
async fn main() {
    let board = Board::run().await;

    board
        .on("ready", |board: Board| async move {
            let mut led = Led::new(board, 13).expect("Embedded led is instantiated");

            // Next is equivalent to this:
            // ```
            // loop {
            //     led.on().unwrap();
            //     tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            //     led.off().unwrap();
            //     tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            // }
            // ```
            led.blink(500).expect("Led should be blink");
        })
        .await;
}
