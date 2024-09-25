use hermes_five::{Board, pause};
use hermes_five::devices::{Actuator, Led};

#[hermes_five::runtime]
async fn main() {
    let board = Board::run();

    board.on("ready", |board: Board| async move {
        // Register a LED on pin 13 (default arduino led).
        let mut led = Led::new(&board, 13).expect("Embedded led is instantiated");

        // Pulse the LED every 500ms.
        // @todo create pulse helper
        // led.pulse(100);

        // Wait for 10 seconds.
        pause!(10000);

        // stop() stops the animation.
        // off() shuts the led off.
        led.stop();
        led.off()?;

        Ok(())
    });
}
