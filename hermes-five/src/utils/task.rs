//! Defines Hermes-Five Runtime task runner.
use std::future::Future;

use anyhow::Result;
use futures::TryFutureExt;
use tokio::sync::{Mutex, OnceCell};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio::task::JoinHandle;

use crate::errors::{Error, RuntimeError, Unknown};

/// Represents the result of a TaskResult.
/// A task may return either () or Result<(), Error> for flexibility which
/// will be converted to TaskResult sent to the runtime..
pub enum TaskResult {
    Ok,
    Err(anyhow::Error),
}

/// Represents an arc protected handler for a task.
pub type TaskHandler = JoinHandle<Result<(), Error>>;

/// Globally accessible runtime transmitter(TX)/receiver(RX) (not initialised yet)
pub static RUNTIME_TX: OnceCell<Mutex<Option<Sender<Receiver<TaskResult>>>>> =
    OnceCell::const_new();
pub static RUNTIME_RX: OnceCell<Mutex<Option<Receiver<Receiver<TaskResult>>>>> =
    OnceCell::const_new();

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

pub async fn init_task_channel() {
    // If no receiver is configured, create a new one (with associated sender).
    RUNTIME_RX
        .get_or_init(|| async {
            // Arbitrary limit to 100 simultaneous tasks.
            let (tx, rx) = tokio::sync::mpsc::channel::<Receiver<TaskResult>>(100);

            // Set the runtime sender.
            RUNTIME_TX
                .get_or_init(|| async { Mutex::new(Some(tx)) })
                .await;

            // Set the runtime receiver.
            Mutex::new(Some(rx))
        })
        .await;
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
///     task::run(async move {
///         // whatever
///     }).await;
/// }
/// ```
pub async fn run<F, T>(future: F) -> Result<TaskHandler, Error>
where
    F: Future<Output = T> + Send + 'static,
    T: Into<TaskResult> + Send + 'static,
{
    // Create a transmitter(tx)/receiver(rx) unique to this task.
    let (task_tx, task_rx) = tokio::sync::mpsc::channel(1);

    // --
    // Create a task to run our future: note how we capture the tx...
    let handler = task::spawn(async move {
        // ...to send the result of the future through that channel.
        let result = future.await.into();
        task_tx.send(result).await.map_err(|err| Unknown {
            info: err.to_string(),
        })?;
        Ok(())
    });

    // --
    // Send the receiver(rx) side of the task-channel to the runtime.

    let cell = RUNTIME_TX.get().ok_or(RuntimeError)?;
    let mut lock = cell.lock().await;
    let runtime_tx = lock.as_mut().ok_or(RuntimeError)?;

    runtime_tx
        .send(task_rx)
        .map_err(|err| Unknown {
            info: err.to_string(),
        })
        .await?;

    Ok(handler)
}

#[macro_export]
macro_rules! pause {
    ($ms:expr) => {
        tokio::time::sleep(tokio::time::Duration::from_millis($ms as u64)).await
    };
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::time::SystemTime;

    use anyhow::bail;

    use crate::errors::Error;
    use crate::utils::task;

    #[hermes_macros::runtime]
    async fn my_runtime() -> Result<(), Error> {
        task::run(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            task::run(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                task::run(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                })
                .await?;
                Ok(())
            })
            .await?;
            Ok(())
        })
        .await?;

        task::run(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        })
        .await?;

        task::run(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        })
        .await?;

        Ok(())
    }

    #[test]
    fn test_task_parallel_execution() {
        // Tasks should be parallel and function should be blocked until all done.
        // Therefore the `my_runtime()` function should take more time than the longest task, but less
        // than the sum of task times.
        let start = SystemTime::now();
        my_runtime().unwrap();
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
    async fn test_task_abort_execution() {
        let flag = Arc::new(AtomicU8::new(0));
        let flag_clone = flag.clone();

        // Increment the flag after 100ms
        task::run(async move {
            pause!(100);
            flag_clone.fetch_add(1, Ordering::SeqCst);
        })
        .await
        .expect("Should not panic");

        // The flag should not have been incremented before the 100ms elapsed.
        pause!(50);
        assert_eq!(
            flag.load(Ordering::SeqCst),
            0,
            "Flag should not be updated by the task before 100ms",
        );

        // The flag should have been incremented after the 100ms elapsed.
        pause!(100);
        assert_eq!(
            flag.load(Ordering::SeqCst),
            1,
            "Flag should be updated by the task after 100ms",
        );

        // ######################
        // Same test but aborting
        let flag_clone = flag.clone();

        // Increment the flag after 100ms
        let handler = task::run(async move {
            pause!(100);
            flag_clone.fetch_add(1, Ordering::SeqCst);
        })
        .await
        .expect("Should not panic");

        // The flag should not have been incremented before the 100ms elapsed.
        pause!(50);
        assert_eq!(
            flag.load(Ordering::SeqCst),
            1,
            "Flag should not be updated by the task before 100ms",
        );

        // Abort the task
        handler.abort();

        // The flag should not have been incremented after the 100ms elapsed.
        pause!(100);
        assert_eq!(
            flag.load(Ordering::SeqCst),
            1,
            "Flag should be updated by the task after 100ms",
        );
    }

    #[hermes_macros::test]
    async fn test_task_with_result() {
        let task = task::run(async move { Ok(()) });

        assert!(
            task.await.is_ok(),
            "An Ok(()) task do not panic the runtime"
        );

        let task = task::run(async move {
            bail!("Wow panic!");
        });

        assert!(
            task.await.is_ok(),
            "A panicking task do not panic the runtime"
        );
    }
}
