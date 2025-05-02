//! Defines Hermes-Five Runtime task runner.
use std::future::Future;
use std::panic::AssertUnwindSafe;

use futures::FutureExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::{task, task_local};

use crate::errors::{Error, RuntimeError, UnknownError};

/// Represents the result of a TaskResult.
/// A task may return either () or Result<(), Error> for flexibility which
/// will be converted to TaskResult sent to the runtime.
pub enum TaskResult {
    Ok,
    Err(Error),
}

/// Represents an arc protected handler for a task.
pub type TaskHandler = JoinHandle<Result<(), Error>>;

impl From<Result<(), Error>> for TaskResult {
    fn from(result: Result<(), Error>) -> Self {
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

pub fn setup_rt(test: bool) -> Runtime {
    let mut builder = if test {
        tokio::runtime::Builder::new_current_thread()
    } else {
        let mut b = tokio::runtime::Builder::new_multi_thread();
        b.worker_threads(4);
        b
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
        TASK.try_with(|t| t.clone()).map_err(|_| RuntimeError)
    }

    /// Runs a future within the current task context.
    async fn catch_errors<F, T>(self, future: F) -> Result<(), Error>
    where
        F: Future<Output = T> + Send + 'static,
        T: Into<TaskResult> + Send + 'static,
    {
        // allow ourselves to catch panics
        let future = AssertUnwindSafe(future).catch_unwind();

        // run our future within the scope our task queue.
        let mut task = std::pin::pin!(TASK.scope(self, future));

        // await the future response.
        let res = task.as_mut().await;

        // get back our task queue.
        let queue = task.take_value().ok_or(RuntimeError)?;

        // check for a panic.
        let res: TaskResult = match res {
            Ok(res) => res.into(),
            Err(panic) => {
                // ignore errors if receiver is missing.
                _ = queue.results.send(UnknownError {
                    info: "task panicked".to_string(),
                });

                // continue the panic.
                std::panic::resume_unwind(panic);
            }
        };

        // send error, if there was one.
        if let TaskResult::Err(e) = res {
            queue.results.send(e).map_err(|_| RuntimeError)?;
        }

        Ok(())
    }

    /// Create a new task context, run a task inside it,
    /// and wait for all tasks to complete.
    async fn run<F: Future>(f: F) -> F::Output {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let task = Self { results: tx };

        // run our future within the scope our task queue.
        let res = TASK.scope(task, f).await;

        // wait for tasks to complete.
        // recv returns None if all corresponding senders are dropped.
        while let Some(err) = rx.recv().await {
            log::error!("Task failed: {:?}", err.to_string());
            eprintln!("Task failed: {:?}", err.to_string());
        }

        res
    }
}

pub struct Runtime {
    runtime: tokio::runtime::Runtime,
}

impl Runtime {
    pub fn block_on<F: Future>(&self, f: F) -> F::Output {
        self.runtime.block_on(TaskRegistration::run(f))
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
    T: Into<TaskResult> + Send + 'static,
{
    let task = TaskRegistration::get()?;

    // Create a task to run our future: note how we capture `task`.
    // This `task` acts as a token to keep track of active tasks.
    Ok(task::spawn(async move { task.catch_errors(future).await }))
}

#[macro_export]
macro_rules! pause {
    ($ms:expr) => {
        tokio::time::sleep(tokio::time::Duration::from_millis($ms as u64)).await
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

    use serial_test::serial;

    use crate::errors::{Error, UnknownError};
    use crate::utils::task;

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
            flag_clone.fetch_add(1, Ordering::SeqCst);
        })
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

        // ########################################
        // Same test but aborting
        let flag_clone = flag.clone();

        // Increment the flag after 100ms
        let handler = task::run(async move {
            pause!(100);
            flag_clone.fetch_add(1, Ordering::SeqCst);
        })
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

    #[hermes_five_macros::test]
    async fn test_task_with_result() {
        let task = task::run(async move { Ok(()) });

        assert!(task.is_ok(), "An Ok(()) task do not panic the runtime");

        let task = task::run(async move {
            Err(UnknownError {
                info: "wow panic!".to_string(),
            })
        });

        assert!(task.is_ok(), "A panicking task do not panic the runtime");
    }
}
