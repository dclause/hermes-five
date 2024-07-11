use hermes_five::Board;

#[hermes_macros::runtime]
async fn main() {
    let board = Board::default();

    // Something long happening before we can register our events.
    hermes_five::utils::sleep(std::time::Duration::from_secs(1)).await;

    board
        .on("ready", |_: Board| async move {
            for i in 1..5 {
                println!("Callback 1: do something #{}", i);
                hermes_five::utils::sleep(std::time::Duration::from_millis(500)).await;
            }
        })
        .await;

    board
        .on("ready", |_: Board| async move {
            for i in 1..5 {
                println!("Callback 2: do another #{}", i);
                hermes_five::utils::sleep(std::time::Duration::from_millis(500)).await;
            }
        })
        .await;

    board
        .on("ready", |board: Board| async move {
            println!("Callback 3: close board in 1sec");
            hermes_five::utils::sleep(std::time::Duration::from_secs(1)).await;
            board.close().await;
        })
        .await;

    board
        .on("close", |_: Board| async move {
            println!("Connection closed on board.");
        })
        .await;

    // The trick is to start the board only when everything is defined.
    board.open().await;
}
