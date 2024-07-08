use hermes_five::entities::Board;

#[hermes_macros::runtime]
async fn main() {
    // ############
    // Solution 1:
    // Register all events on a single board.
    // Open that board afterward, so the copy are done when all callback exists.
    // Explanation: The board within the callback is a copy of the board as it exists at the moment the event
    // is triggered, since everything is asynchronous, we must ensure that copy is done AFTER the
    // close event registration, hence run `.open()` at the end.

    // The default method does not yet run the board.
    let board1 = Board::default();

    board1
        // The board within the callback is a copy of the board as it exists at the moment the event
        // is triggered, since everything is asynchronous, we must ensure that copy is done AFTER the
        // close event registration, hence run `.open()` at the end.
        .on("ready", |copy: Board| async move {
            println!("Connection done on board1");
            hermes_five::utils::sleep(std::time::Duration::from_secs(1)).await;
            // Copy owns a "close" callback at this point.
            copy.close().await;
        })
        .await;

    // The trick is here to register all events before the board.open() method.
    board1
        .on("close", |_: Board| async move {
            println!("Connection closed on board1");
        })
        .await;

    board1.open().await;

    // ############
    // Solution 2:
    // Register the close event first, then the "ready" event.
    let board2 = Board::run().await;
    board2
        .on("close", |_: Board| async move {
            println!("Connection closed on board2");
        })
        .await;
    board2
        .on("ready", |copy: Board| async move {
            println!("Connection done on board2");
            hermes_five::utils::sleep(std::time::Duration::from_secs(1)).await;
            copy.close().await;
        })
        .await;

    // ############
    // Solution 3:
    // Assign the callback to the exact board "copy" you need.
    let board3 = Board::run().await;
    board3
        .on("ready", |copy: Board| async move {
            println!("Connection done on board3");

            copy.on("close", |_: Board| async move {
                println!("Connection closed on board3");
            })
            .await;

            hermes_five::utils::sleep(std::time::Duration::from_secs(1)).await;
            copy.close().await;
        })
        .await;
}
