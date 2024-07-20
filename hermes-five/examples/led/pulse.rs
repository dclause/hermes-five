use hermes_five::{Board, pause};
use hermes_five::devices::Led;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run().await;

    board
        .on("ready", |board: Board| async move {
            // Register a LED on pin 13 (default arduino led).
            let mut led = Led::new(&board, 13).expect("Embedded led is instantiated");
            println!("{:?}", led.pin());

            // Pulse the LED every 500ms.
            led.pulse(100).await;

            // Wait for 10 seconds.
            pause!(10000);

            // stop() stops the animation.
            // off() shuts the led off.
            led.stop().off();
        })
        .await;
}
