use hermes_five::{Board, pause};
use hermes_five::devices::Led;

#[hermes_macros::runtime]
async fn main() {
    let board = Board::run().await;

    board
        .on("ready", |board: Board| async move {
            // Register a LED on pin 13 (default arduino led).
            let mut led = Led::new(&board, 13).expect("Embedded led is instantiated");
            println!("{:?}", led.pin());

            // Blinks the LED every 100ms (for 3sec).
            led.blink(100).await;

            // Notice how blink is not blocker for the current thread, yet it is for the runtime
            println!("This will print immediately");
            pause!(3000);
            println!("This will print 3 seconds later");

            // Stops the LED animation.
            led.stop();

            // Because
        })
        .await;

    // board
    //     .on("ready", |board: Board| async move {
    //         // Register a LED on pin 11.
    //         let mut led = Led::new(&board, 11)
    //             .expect("Embedded led is instantiated")
    //             // Lower intensity: this will now impose a PWM compatible pin.
    //             .with_intensity(1)
    //             .unwrap();
    //         println!("{:?}", led.pin());
    //
    //         loop {
    //             led.on().unwrap();
    //             tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    //             led.off().unwrap();
    //             tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    //         }
    //
    //         // led.blink(500).expect("Led should be blink");
    //     })
    //     .await;
}
