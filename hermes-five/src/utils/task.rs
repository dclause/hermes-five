//! Defines Hermes-Five Runtime task runner.

use std::future::Future;

use anyhow::anyhow;
use tokio::sync::{OnceCell, RwLock};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio::task::JoinHandle;

/// Globally accessible runtime sender.
pub(crate) static SENDER: OnceCell<RwLock<Option<Sender<JoinHandle<()>>>>> = OnceCell::const_new();
pub(crate) static RECEIVER: OnceCell<RwLock<Option<Receiver<JoinHandle<()>>>>> =
    OnceCell::const_new();

/// Runs a given future as a Tokio task while ensuring the main function (marked by `#[hermes_five::runtime]`)
/// will not finish before all tasks running as done.
/// This is done by using a globally accessible channel to communicate the handlers to be waited by the
/// runtime.
///
/// # Parameters
/// * `future`: A future that implements `Future<Output = ()>`, `Send`, and has a `'static` lifetime.
///
/// # Errors
/// Returns an error if the lock cannot be acquired or if the sender is not initialized or if sending the task handle fails.
///
/// # Example
/// ```
/// #[hermes_five::runtime]
/// async fn main() {
///     // whatever
/// }
/// ```
pub async fn run<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    let lock = SENDER.get().unwrap().read().await;
    let sender = lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tasks transmitter not initialized"))
        .unwrap();
    let handle = task::spawn(future);
    match sender.send(handle).await {
        Ok(_) => {}
        Err(e) => panic!("Task handler not sent: {}", e),
    };
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::utils::task;

    #[hermes_macros::runtime]
    async fn my_runtime() {
        task::run(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            task::run(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                task::run(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                })
                .await;
            })
            .await;
        })
        .await;

        task::run(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        })
        .await;
    }

    #[test]
    fn test_task_execution() {
        // Tasks should be parallel and function should be blocked until all done.
        // Therefore the `my_runtime()` function should take more time than the longest task, but less
        // than the sum of task times.
        let start = SystemTime::now();

        my_runtime();

        let end = SystemTime::now();

        let duration = end.duration_since(start).unwrap().as_millis();
        assert!(
            duration > 300,
            "Duration should be greater than 300ms (found: {})",
            duration
        );
        assert!(
            duration < 350,
            "Duration should be lower than 350ms (found: {})",
            duration
        );
    }
}
