//! Defines Hermes-Five Runtime task runner.

use std::future::Future;

use anyhow::{anyhow, Result};
use tokio::sync::{OnceCell, RwLock};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio::task::JoinHandle;

/// Globally accessible runtime sender.
pub static SENDER: OnceCell<RwLock<Option<Sender<JoinHandle<TaskResult>>>>> = OnceCell::const_new();
pub static RECEIVER: OnceCell<RwLock<Option<Receiver<JoinHandle<TaskResult>>>>> =
    OnceCell::const_new();

/// Represents the result of a TaskResult.
/// A task may return either () or Result<(), Error> for flexibility which
/// will be converted to TaskResult sent to the runtime..
pub enum TaskResult {
    Ok,
    Err(anyhow::Error),
}

impl From<Result<()>> for TaskResult {
    fn from(result: Result<()>) -> Self {
        match result {
            Ok(_) => TaskResult::Ok,
            Err(e) => TaskResult::Err(e),
        }
    }
}

impl From<()> for TaskResult {
    fn from(_: ()) -> Self {
        TaskResult::Ok
    }
}

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
pub async fn run<F, T>(future: F)
where
    F: Future<Output = T> + Send + 'static,
    T: Into<TaskResult> + Send + 'static,
{
    let lock = SENDER.get().unwrap().read().await;
    let sender = lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tasks transmitter not initialized"))
        .unwrap();
    let handle = task::spawn(async move { future.await.into() });
    match sender.send(handle).await {
        Ok(_) => {}
        Err(e) => panic!("Task handler not sent: {}", e),
    };
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use anyhow::bail;

    use crate::utils::task;

    #[hermes_macros::runtime]
    async fn my_runtime() {
        task::run(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
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
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        })
        .await;

        task::run(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
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
            duration > 500,
            "Duration should be greater than 500ms (found: {})",
            duration,
        );
        assert!(
            duration < 1500,
            "Duration should be lower than 1500ms (found: {})",
            duration,
        );
    }

    #[hermes_macros::test]
    async fn test_task_with_result() {
        let task = task::run(async move { Ok(()) });

        assert_eq!(task.await, (), "An Ok(()) task do not panic the runtime");

        let task = task::run(async move {
            bail!("Wow panic!");
        });

        assert_eq!(task.await, (), "A panicking task do not panic the runtime");
    }
}
