use hermes_five::Board;
use hermes_five::devices::Led;

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board
        .on("ready", |board: Board| async move {
            // Register a LED on pin 11.
            let mut led = Led::new(&board, 11)
                .unwrap()
                // Lower intensity to 50%: this will now impose a PWM compatible pin.
                .with_intensity(50)
                .unwrap();

            led.blink(500).await;
        })
        .await;
}
