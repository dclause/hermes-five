//! Defines Hermes-Five event manager system.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::future::BoxFuture;
use futures::FutureExt;
use tokio::sync::Mutex;

use crate::utils::task;

type SyncedCallbackMap = Mutex<HashMap<String, Vec<CallbackWrapper>>>;
pub type EventHandler = usize;

#[derive(Clone, Default)]
pub struct EventManager {
    callbacks: Arc<SyncedCallbackMap>,
    next_id: Arc<AtomicUsize>,
}

struct CallbackWrapper {
    id: EventHandler,
    callback: Box<dyn FnMut(Arc<dyn Any + Send + Sync>) -> BoxFuture<'static, ()> + Send>,
}

impl EventManager {
    /// Register event handler for a specific event name.
    ///
    /// # Parameters
    /// * `event` - The event name (any type that matches an Into<String>)
    /// * `callback` - An async moved callback that accepts a single parameter as an argument.
    ///                The argument can be anything that might be both `Send + Sync`.
    ///                You can trick multiple parameters by turning them in a single tuple.
    ///
    /// # Return
    /// Returns an EventHandler that can be used by the `unregister()` method.
    ///
    /// # Errors
    /// If the event handler does not match the expected emitted event exactly it will fail silently.
    /// That means if the tuple gave in the callback parameter does not exactly match the emit one
    /// no handler will be called.
    ///
    /// # Example
    ///
    /// ```
    /// // Instantiate an EventManager
    /// let events: EventManager = Default::default();
    ///
    /// // Register various handlers for the same event.
    /// events.on("ready", |name: String| async move { })
    /// events.on("ready", |age: u8| async move { })
    /// events.on("ready", |whatever: Vec<[u8;4]>| async move { })
    /// events.on("ready", |(name, age): (&str, u8)| async move {
    ///     println!("Event handler with parameters: {} {}.", name, age);
    ///     std::thread::sleep(std::time::Duration::from_secs(1));
    ///     println!("Event handler done");
    /// });
    ///
    /// // Invoke handlers for "ready" event.
    /// events.emit("ready", ("foo", 69u8)).await;
    ///
    /// // No matching handler (because of parameters) will be called
    /// events.emit("ready", ("bar")).await;
    /// ```
    pub async fn on<S, F, T, Fut>(&self, event: S, mut callback: F) -> EventHandler
    where
        S: Into<String>,
        T: 'static + Send + Sync + Clone,
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let event_name = event.into();
        let callback_event = event_name.clone();
        // Generate a unique ID.
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        // Boxes the callback and downcast its parameter.
        let boxed_callback = Box::new(move |arg: Arc<dyn Any + Send + Sync>| {
            match arg.downcast::<T>() {
                Ok(arg) => (callback)((*arg).clone()).boxed(),
                Err(_) => {
                    // Handle error case where the argument is not of type T
                    log::warn!("The callback for event '{}' could not be called because parameter does not match", callback_event);
                    // Current strategy is to ignore the callback
                    Box::pin(async {})
                }
            }
        });

        let wrapper = CallbackWrapper {
            id,
            callback: boxed_callback,
        };

        self.callbacks
            .lock()
            .await
            .entry(event_name)
            .or_default()
            .push(wrapper);

        id
    }

    /// Invoke all event handlers registered for a specific event name.
    /// Only the callback registered by the `on()` method and whose payload matches the declared
    /// callback type will be called. All others will be silently skipped.
    ///
    /// # Parameters
    /// * `event`:  The event name (any type that matches an `Into<String>`)
    /// * `payload`: The event payload (must be `'static + Send + Sync`)
    ///              The payload can be anything that might be both `Send + Sync`.
    ///              You can trick multiple parameters by turning them in a single tuple.
    ///
    /// # Example
    ///
    /// ```
    /// // Instantiate an EventManager
    /// let events: EventManager = Default::default();
    ///
    /// // Register various handlers for the same event.
    /// events.on("ready", |name: &str| async move {
    ///     println!("Callback 1");
    /// })
    /// events.on("ready", |age: u8| async move {
    ///     println!("Callback 2");
    /// })
    ///
    /// // Invoke handlers for "ready" event matching &str parameter.
    /// events.emit("ready", "foo").await;
    /// // Invoke handlers for "ready" event matching u8 parameter.
    /// events.emit("ready", 42).await;
    ///
    /// // No event registered for "nothing" event.
    /// events.emit("nothing", ()).await;
    /// ```
    pub async fn emit<'a, S, T>(&self, event: S, payload: T)
    where
        S: Into<String>,
        T: 'static + Send + Sync,
    {
        let payload_any: Arc<dyn Any + Send + Sync> = Arc::new(payload);
        if let Some(callbacks) = self.callbacks.lock().await.get_mut(&event.into()) {
            for wrapper in callbacks.iter_mut() {
                let payload_clone = payload_any.clone();
                let future = (wrapper.callback)(payload_clone);
                task::run(future).await;
            }
        }
    }

    /// Unregister a given handler if found.
    ///
    /// # Example
    ///
    /// ```
    /// // Instantiate an EventManager
    /// let events: EventManager = Default::default();
    ///
    /// // Register various handlers for the same event.
    /// let handler1 = events.on("ready", |age: u8| async move {
    ///     println!("Callback 1");
    /// })
    /// let handler2 = events.on("ready", |age: u8| async move {
    ///     println!("Callback 2");
    /// })
    ///
    /// // Unregister handler 1
    /// events.unregister(handler);
    ///
    /// // Invoke handlers for "ready" event matching u8 parameter.
    /// // Only the callback2 remains to be called here.
    /// events.emit("ready", 42).await;
    /// ```
    pub async fn unregister(&self, handler: EventHandler) {
        let _ = &self
            .callbacks
            .lock()
            .await
            .values_mut()
            .for_each(|v| v.retain(|cb| cb.id != handler));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicU8};

    use super::*;

    #[hermes_macros::test]
    async fn test_register_and_emit_event() {
        let events: EventManager = Default::default();
        let payload = Arc::new(AtomicBool::new(false));

        events
            .on("register", |flag: Arc<AtomicBool>| async move {
                flag.store(true, Ordering::SeqCst);
            })
            .await;

        events.emit("register", payload.clone()).await;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert!(
            payload.load(Ordering::SeqCst),
            "The flag have been set by the triggered event."
        );
    }

    #[hermes_macros::test]
    async fn test_unregister_event_handler() {
        let events: EventManager = Default::default();
        let flag = Arc::new(AtomicBool::new(false));

        let handler = events
            .on("unregister", |flag: Arc<AtomicBool>| async move {
                flag.store(true, Ordering::SeqCst);
            })
            .await;

        events.unregister(handler).await;
        events.emit("unregister", flag.clone()).await;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert!(
            !flag.load(Ordering::SeqCst),
            "The event was unregistered: the flag have not been set."
        );
    }

    #[hermes_macros::test]
    async fn test_multiple_handlers() {
        let events: EventManager = Default::default();
        let flag = Arc::new(AtomicUsize::new(0));

        events
            .on("multiple", |flag: Arc<AtomicUsize>| async move {
                let value = flag.load(Ordering::SeqCst);
                flag.store(value + 1, Ordering::SeqCst);
            })
            .await;

        events
            .on("multiple", |flag: Arc<AtomicUsize>| async move {
                let value = flag.load(Ordering::SeqCst);
                flag.store(value + 1, Ordering::SeqCst);
            })
            .await;

        events
            .on(
                "multiple",
                |(_not_matching, flag): (u8, Arc<AtomicUsize>)| async move {
                    let value = flag.load(Ordering::SeqCst);
                    flag.store(value + 1, Ordering::SeqCst);
                },
            )
            .await;

        events.emit("multiple", flag.clone()).await;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(
            flag.load(Ordering::SeqCst),
            2,
            "The flag have been increased by 2."
        );
    }

    #[hermes_macros::test]
    async fn test_event_with_complex_payload() {
        let events: EventManager = Default::default();
        let flag = Arc::new(AtomicU8::new(0));

        events
            .on(
                "payload",
                |(number1, number2, container): (u8, u8, Arc<AtomicU8>)| async move {
                    container.store(number1 + number2, Ordering::SeqCst);
                },
            )
            .await;
        events.emit("payload", (42u8, 69u8, flag.clone())).await;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert_eq!(
            flag.load(Ordering::SeqCst),
            111,
            "The complex flag has been properly received."
        );
    }

    #[hermes_macros::test]
    async fn test_no_handlers_for_event() {
        let events: EventManager = Default::default();
        let result = events.emit("no_event", ()).await;
        assert_eq!(result, (), "Nothing to do.");
    }
}
