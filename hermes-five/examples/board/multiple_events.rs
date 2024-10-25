use hermes_five::hardware::{Board, BoardEvent};
use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    let board = Board::default();

    // Something long happening before we can register our events.
    pause!(1000);

    board.on(BoardEvent::OnReady, |_: Board| async move {
        for i in 1..5 {
            println!("Callback 1: do something #{}", i);
            pause!(500);
        }

        Ok(())
    });

    board.on(BoardEvent::OnReady, |_: Board| async move {
        for i in 1..5 {
            println!("Callback 2: do another #{}", i);
            pause!(500);
        }

        Ok(())
    });

    board.on(BoardEvent::OnReady, |board: Board| async move {
        println!("Callback 3: close board in 1sec");
        pause!(1000);
        board.close();

        Ok(())
    });

    board.on(BoardEvent::OnClose, |_: Board| async move {
        println!("Connection closed on board.");
        Ok(())
    });

    // The trick is to start the board only when everything is defined.
    board.open();
}
