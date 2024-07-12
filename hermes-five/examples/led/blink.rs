use hermes_five::Board;
use hermes_five::devices::Led;

#[hermes_macros::runtime]
async fn main() {
    let board = Board::run().await;

    board
        .on("ready", |board: Board| async move {
            let mut led = Led::new(board.clone(), 13).expect("Embedded led is instantiated");
            println!("13- Pin 11 {}", board.pins.get(11).unwrap().mode.id);
            println!("13- Pin 13 {}", board.pins.get(13).unwrap().mode.id);

            loop {
                led.on().unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                led.off().unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        })
        .await;

    board
        .on("ready", |board: Board| async move {
            let mut led = Led::new(board.clone(), 11).expect("Embedded led is instantiated");
            led.with_intensity(50).unwrap();

            println!("11- Pin 11 {}", board.pins.get(11).unwrap().mode.id);
            println!("11- Pin 13 {}", board.pins.get(13).unwrap().mode.id);

            loop {
                led.on().unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                led.off().unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            // led.blink(500).expect("Led should be blink");
        })
        .await;
}
