//! Defines Hermes-Five Runtime task runner.
use std::cell::RefCell;
use std::future::Future;
use std::sync::Arc;

use futures::{stream::FuturesUnordered, StreamExt};
use parking_lot::RwLock;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::task;
use tokio::task::JoinHandle;

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

type Task = oneshot::Receiver<TaskResult>;
type Queue = Arc<RwLock<FuturesUnordered<Task>>>;

thread_local! {
    static TASKS: RefCell<Option<Queue>> = const { RefCell::new(None) };
}

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

pub fn setup_rt(test: bool) -> TaskQueueConsumer {
    let task_queue = Arc::new(RwLock::new(FuturesUnordered::new()));

    let mut builder = if test {
        tokio::runtime::Builder::new_current_thread()
    } else {
        let mut b = tokio::runtime::Builder::new_multi_thread();
        b.worker_threads(4);
        b
    };

    let queue = task_queue.clone();
    builder.enable_all().on_thread_start(move || {
        init_thread(queue.clone());
    });

    TaskQueueConsumer {
        runtime: builder.build().unwrap(),
        queue: task_queue,
    }
}

fn init_thread(queue: Queue) -> Option<Queue> {
    TASKS.with_borrow_mut(|t| std::mem::replace(t, Some(queue)))
}

pub struct TaskQueueConsumer {
    runtime: Runtime,
    queue: Queue,
}

impl TaskQueueConsumer {
    pub fn block_on<F: Future>(&self, f: F) -> F::Output {
        struct ResetQueue(Option<Queue>);
        impl Drop for ResetQueue {
            fn drop(&mut self) {
                if let Some(queue) = self.0.take() {
                    init_thread(queue);
                } else {
                    TASKS.with_borrow_mut(|t| t.take());
                }
            }
        }

        let _guard = ResetQueue(init_thread(self.queue.clone()));
        self.runtime.block_on(async {
            let res = f.await;

            // wait for tasks to complete.
            futures::stream::poll_fn(|cx| self.queue.write().poll_next_unpin(cx))
                .for_each(|task_result| async {
                    match task_result {
                        Ok(TaskResult::Ok) => {}
                        Err(_) => {
                            log::error!("Task aborted");
                            eprintln!("Task aborted");
                        }
                        Ok(TaskResult::Err(err)) => {
                            log::error!("Task failed: {:?}", err.to_string());
                            eprintln!("Task failed: {:?}", err.to_string());
                        }
                    }
                })
                .await;

            res
        })
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
    // Create a transmitter(tx)/receiver(rx) unique to this task.
    let (task_tx, task_rx) = oneshot::channel();

    // --
    // Create a task to run our future: note how we capture the tx...
    let handler = task::spawn(async move {
        // ...to send the result of the future through that channel.
        let result = future.await.into();
        task_tx
            .send(result)
            .map_err(|_result: TaskResult| UnknownError {
                info: "task receiver was closed".to_string(),
            })
    });

    TASKS.with_borrow(|r| {
        r.as_deref()
            .ok_or(RuntimeError)
            .map(|tasks| tasks.read().push(task_rx))
    })?;

    Ok(handler)
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
