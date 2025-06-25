//! Defines Hermes-Five Runtime task runner.
use std::future::Future;
use std::panic::AssertUnwindSafe;

use futures::FutureExt;
use log::error;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::{task, task_local};

use crate::errors::{Error, RuntimeError};
use crate::utils::GenericResult;

/// Represents an arc protected handler for a task.
pub type TaskHandler = JoinHandle<Result<(), Error>>;

pub fn setup_rt(test: bool) -> Runtime {
    let mut builder = if test {
        tokio::runtime::Builder::new_current_thread()
    } else {
        tokio::runtime::Builder::new_multi_thread()
    };

    Runtime {
        runtime: builder.enable_all().build().unwrap(),
    }
}

task_local! {
    /// The current task registration.
    static TASK: TaskRegistration;
}

#[derive(Clone)]
struct TaskRegistration {
    // Tasks will not send anything if they exited without error.
    // Tasks will not send anything if they were explicitly aborted.
    // Tasks will send an error if they exited with error or panicked.
    results: mpsc::UnboundedSender<Error>,
}

impl TaskRegistration {
    /// Get a handle to the current task context.
    fn get() -> Result<Self, Error> {
        TASK.try_with(|t| t.clone()).map_err(|e| RuntimeError {
            cause: format!(
                "{} (you probably forgot `#[hermes_five::runtime]` annotation)",
                e
            )
            .to_string(),
        })
    }

    /// Runs a future within the current task context.
    async fn catch_errors<F, T>(self, future: F) -> Result<(), Error>
    where
        F: Future<Output = T> + Send + 'static,
        T: Into<GenericResult> + Send + 'static,
    {
        // Allow ourselves to catch panics.
        let future = AssertUnwindSafe(future).catch_unwind();

        // Run our future within the scope our task queue.
        let mut task = std::pin::pin!(TASK.scope(self, future));

        // Await the future response.
        let res = task.as_mut().await;

        // Get back our task queue.
        let queue = task.take_value().unwrap();

        // Check for a panic.
        match res {
            Err(panic) => {
                queue
                    .results
                    .send(RuntimeError {
                        cause: "Task panicked".to_string(),
                    })
                    .unwrap();
                // Continue the panic.
                std::panic::resume_unwind(panic);
            }
            Ok(res) => {
                // Check for an error.
                match res.into() {
                    GenericResult::Ok => Ok(()),
                    GenericResult::Err(err) => {
                        queue.results.send(err.clone()).unwrap();
                        Err(err)
                    }
                }
            }
        }
    }

    /// Create a new task context, run a task inside it,
    /// and wait for all tasks to complete.
    async fn run<F: Future>(f: F) -> F::Output {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let task = Self { results: tx };

        // Run our future within the scope our task queue.
        let res = TASK.scope(task, f).await;

        // Wait for tasks to complete.
        // recv returns None if all corresponding senders are dropped.
        while let Some(err) = rx.recv().await {
            error!("Task failed: {:?}", err.to_string());
            eprintln!("Task failed: {:?}", err.to_string());
        }

        res
    }
}

/// Wraps the tokio Runtime: used to customize `block_on` function call.
pub struct Runtime {
    runtime: tokio::runtime::Runtime,
}

impl Runtime {
    /// Runs a future to completion using the underlying Tokio runtime wrapped as a Task.
    pub fn block_on<F: Future>(&self, f: F) -> F::Output {
        self.runtime.block_on(TaskRegistration::run(f))
    }
}

/// Runs a given future as a Tokio task while ensuring the main function (marked by `#[hermes_five::runtime]`)
/// will not finish before all tasks running as done.
///
/// # Parameters
/// * `future`: A future that implements `Future<Output = ()>`, `Send`, and has a `'static` lifetime.
///
/// # Errors
/// Returns an error if the lock cannot be acquired or if the sender is not initialized or if sending the task handle fails.
///
/// # Example
/// ```
/// use hermes_five::utils::task;
///
/// #[hermes_five::runtime]
/// async fn main() {
///     let handler = task::run(async move {
///         // whatever
///     }).unwrap();
///     // Abort the task early.
///     handler.abort();
/// }
/// ```
pub fn run<F, T>(future: F) -> Result<TaskHandler, Error>
where
    F: Future<Output = T> + Send + 'static,
    T: Into<GenericResult> + Send + 'static,
{
    let task = TaskRegistration::get()?;

    // Create a task to run our future: note how we capture `task`.
    // This `task` acts as a token to keep track of active tasks.
    Ok(task::spawn(async move { task.catch_errors(future).await }))
}

#[macro_export]
macro_rules! pause {
    ($ms:expr) => {
        $crate::tokio::time::sleep($crate::tokio::time::Duration::from_millis($ms as u64)).await
    };
}

#[macro_export]
macro_rules! pause_sync {
    ($ms:expr) => {
        std::thread::sleep(std::time::Duration::from_millis($ms as u64))
    };
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::sync::Arc;
    use std::time::SystemTime;

    use crate::errors::{Error, InternalError};
    use crate::utils::task;
    use serial_test::serial;

    #[hermes_five_macros::runtime]
    async fn my_runtime() -> Result<(), Error> {
        task::run(async move {
            pause!(500);
            task::run(async move {
                pause!(100);
                task::run(async move {
                    pause!(100);
                })?;
                Ok(())
            })?;
            Ok(())
        })?;

        task::run(async move {
            pause!(500);
        })?;

        task::run(async move {
            pause!(500);
        })?;

        Ok(())
    }

    #[serial]
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

    #[hermes_five_macros::test]
    async fn test_task_abort_execution() {
        let flag = Arc::new(AtomicU8::new(0));
        let flag_clone = flag.clone();

        // Increment the flag after 100ms
        task::run(async move {
            pause!(100);
            flag_clone.fetch_add(1, Ordering::Relaxed);
        })
        .expect("Should not panic");

        // The flag should not have been incremented before the 100ms elapsed.
        pause!(50);
        assert_eq!(
            flag.load(Ordering::Relaxed),
            0,
            "Flag should not be updated by the task before 100ms",
        );

        // The flag should have been incremented after the 100ms elapsed.
        pause!(100);
        assert_eq!(
            flag.load(Ordering::Relaxed),
            1,
            "Flag should be updated by the task after 100ms",
        );

        // ########################################
        // Same test but aborting
        let flag_clone = flag.clone();

        // Increment the flag after 100ms
        let handler = task::run(async move {
            pause!(100);
            flag_clone.fetch_add(1, Ordering::Relaxed);
        })
        .expect("Should not panic");

        // The flag should not have been incremented before the 100ms elapsed.
        pause!(50);
        assert_eq!(
            flag.load(Ordering::Relaxed),
            1,
            "Flag should not be updated by the task before 100ms",
        );

        // Abort the task
        handler.abort();

        // The flag should not have been incremented after the 100ms elapsed.
        pause!(100);
        assert_eq!(
            flag.load(Ordering::Relaxed),
            1,
            "Flag should be updated by the task after 100ms",
        );
    }

    #[test]
    fn test_task_with_no_runtime() {
        let task = task::run(async move { Ok(()) });
        assert_eq!(task.unwrap_err().to_string(), "Runtime error: task-local value not set (you probably forgot `#[hermes_five::runtime]` annotation).", "A task does not run outside the runtime");
    }

    #[hermes_five_macros::test]
    async fn test_task_with_result() {
        // Successful task.
        let task = task::run(async move { Ok(()) });
        assert!(task.is_ok(), "An Ok(()) task do not panic the runtime");
        assert!(task.unwrap().await.is_ok(), "The runtime notifies the win");

        // Error task.
        let task = task::run(async move {
            Err(Error::from(InternalError {
                info: "wow error!".to_string(),
            }))
        });
        assert!(task.is_ok(), "A task in error do not panic the runtime");
        let task_error_result = task.unwrap().await.unwrap();
        assert!(
            task_error_result.is_err(),
            "The runtime should catches the error"
        );
        assert_eq!(
            task_error_result.unwrap_err().to_string(),
            "Internal error: wow error!.",
            "The runtime should catches the error"
        );

        // Panicking task.
        let task = task::run(async move {
            panic!("wow panic!");
            #[allow(unreachable_code)]
            ()
        });

        // Await the task: this will trigger the catch_errors logic and error logging.
        assert!(task.is_ok(), "A panicking task do not panic the runtime");
        assert!(
            task.unwrap()
                .await
                .unwrap_err()
                .to_string()
                .contains("wow panic!"),
            "The runtime should catches the panic"
        );
    }
}
