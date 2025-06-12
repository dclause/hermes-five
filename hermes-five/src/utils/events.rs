//! Defines Hermes-Five event manager system.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::errors::Error;
use crate::utils::task;

pub type Result<T> = std::result::Result<T, Error>;
pub type BoxedCallback<T> =
Box<dyn Fn(T) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;


pub type EventHandler = usize;
struct CallbackWrapper<T> {
    id: EventHandler,
    callback: BoxedCallback<T>,
}

#[derive(Clone)]
pub struct EventManager<Ev, T> {
    callbacks: Arc<RwLock<HashMap<Ev, Vec<CallbackWrapper<T>>>>>,
    next_id: Arc<AtomicUsize>,
}

impl<Ev, T> EventManager<Ev, T>
where
    Ev: Eq + std::hash::Hash + Copy + Send + Sync + 'static,
    T: Clone {

    /// Register event handler for a specific event name.
    ///
    /// # Parameters
    /// * `event` - The event name (any type that matches an `Into<String>`)
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
    /// use hermes_five::utils::EventManager;
    /// use hermes_five::pause;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     // Instantiate an EventManager
    ///     let events: EventManager<&str, &str> = Default::default();
    ///
    ///     // Register various handlers for the same event.
    ///     events.on("ready", |data: &str| async move { println!("Callback 1"); Ok(()) });
    ///     events.on("ready", |data: &str| async move { println!("Callback 2"); Ok(()) });
    ///
    ///     // Invoke handlers for "ready" event.
    ///     events.emit("ready", "I am ready!");
    /// }
    /// ```
    pub fn on<F, Fut>(&self, event: Ev, handler: F) -> EventHandler
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {

        // Generate a unique ID.
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        // Boxes the callback
        let boxed_callback: BoxedCallback<T> = Box::new(move |arg: T| Box::pin(handler(arg)));

        // Creates the callback unique wrapper
        let wrapper = CallbackWrapper {
            id,
            callback: boxed_callback,
        };

        let mut lock = self.callbacks.write();
        lock.entry(event).or_default().push(wrapper);

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
    /// use hermes_five::utils::EventManager;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     // Instantiate an EventManager
    ///     let events: EventManager<&str, &str> = Default::default();
    ///
    ///     // Register various handlers for the same event.
    ///     events.on("ready", |data: &str| async move {
    ///         println!("Callback 1");
    ///         Ok(())
    ///     });
    ///     events.on("ready", |data: &str| async move {
    ///         println!("Callback 2");
    ///         Ok(())
    ///     });
    ///
    ///     // Invoke handlers for "ready" event matching &str parameter.
    ///     events.emit("ready", "foo");
    /// }
    /// ```
    pub fn emit(&self, event: Ev, arg: T)
    where
        Self: Sized + Send + Sync + 'static,
    {
        if let Some(wrappers) =  self.callbacks.read().get(&event) {
            for wrapper in wrappers {
                let callback = &wrapper.callback;
                let arg_copy = arg.clone();
                let _ = task::run(callback(arg_copy));
            }
        }
    }

    /// Unregister a given handler if found.
    ///
    /// # Example
    ///
    /// ```
    /// use hermes_five::utils::EventManager;
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     // Instantiate an EventManager
    ///     let events: EventManager<&str, u8> = Default::default();
    ///
    ///     // Register various handlers for the same event.
    ///     let handler1 = events.on("ready", |age: u8| async move {
    ///         println!("Callback 1");
    ///         Ok(())
    ///     });
    ///     let handler2 = events.on("ready", |age: u8| async move {
    ///         println!("Callback 2");
    ///         Ok(())
    ///     });
    ///
    ///     // Unregister handler 1.
    ///     events.unregister(handler1);
    ///
    ///     // Invoke handlers for "ready" event matching u8 parameter.
    ///     // Only the callback2 remains to be called here.
    ///     events.emit("ready", 42);
    /// }
    /// ```
    pub fn unregister(&self, handler: EventHandler) {
        let _ = &self
            .callbacks
            .write()
            .values_mut()
            .for_each(|v| v.retain(|cb| cb.id != handler));
    }
}

impl<Ev, T> Default for EventManager<Ev, T> {
    fn default() -> Self {
        Self {
            callbacks: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(Default::default()),
        }
    }
}

impl<Ev, T> Debug for EventManager<Ev, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.callbacks.read().len() {
            1 => write!(f, "EventManager: 1 registered callback"),
            count => write!(f, "EventManager: {} registered callbacks", count),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicU8};

    use crate::pause;

    use super::*;

    #[hermes_five_macros::test]
    async fn test_register_and_emit_event() {
        let events: EventManager<&str, Arc<AtomicBool>> = Default::default();
        let payload = Arc::new(AtomicBool::new(false));

        events.on("register", |flag: Arc<AtomicBool>| async move {
            flag.store(true, Ordering::SeqCst);
            Ok(())
        });

        events.emit("register", payload.clone());

        pause!(100);
        assert!(
            payload.load(Ordering::SeqCst),
            "The flag have been set by the triggered event."
        );
    }

    #[hermes_five_macros::test]
    async fn test_unregister_event_handler() {
        let events: EventManager<&str, Arc<AtomicBool>> = Default::default();
        let flag = Arc::new(AtomicBool::new(false));

        let handler = events.on("unregister", |flag: Arc<AtomicBool>| async move {
            flag.store(true, Ordering::SeqCst);
            Ok(())
        });

        events.unregister(handler);
        events.emit("unregister", flag.clone());

        pause!(100);
        assert!(
            !flag.load(Ordering::SeqCst),
            "The event was unregistered: the flag have not been set."
        );
    }

    #[hermes_five_macros::test]
    async fn test_multiple_handlers() {
        let events: EventManager<&str, Arc<AtomicUsize>> = Default::default();
        let flag = Arc::new(AtomicUsize::new(0));

        let callback = |flag: Arc<AtomicUsize>| async move {
            let value = flag.load(Ordering::SeqCst);
            flag.store(value + 1, Ordering::SeqCst);
            Ok(())
        };

        events.on("multiple", callback);
        events.on("multiple", callback);

        events.emit("multiple", flag.clone());

        pause!(500);
        assert_eq!(
            flag.load(Ordering::SeqCst),
            2,
            "The flag have been increased by 2."
        );
    }

    #[hermes_five_macros::test]
    async fn test_event_with_complex_payload() {
        let events: EventManager<&str, (u8,u8,Arc<AtomicU8>)> = Default::default();
        let flag = Arc::new(AtomicU8::new(0));

        events.on(
            "payload",
            |(number1, number2, container): (u8, u8, Arc<AtomicU8>)| async move {
                container.store(number1 + number2, Ordering::SeqCst);
                Ok(())
            },
        );
        events.emit("payload", (42u8, 69u8, flag.clone()));

        pause!(100);
        assert_eq!(
            flag.load(Ordering::SeqCst),
            111,
            "The complex flag has been properly received."
        );
    }

    #[hermes_five_macros::test]
    async fn test_no_handlers_for_event() {
        let events: EventManager<&str, ()> = Default::default();
        let result = events.emit("no_event", ());
        assert_eq!(result, (), "Nothing to do.");
    }

    #[test]
    fn test_event_manager_debug() {
        let events: EventManager<&str, ()> = Default::default();
        events.on("test", |_: ()| async move { Ok(()) });
        assert_eq!(
            format!("{:?}", events),
            "EventManager: 1 registered callback"
        );
        events.on("test2", |_: ()| async move { Ok(()) });
        assert_eq!(
            format!("{:?}", events),
            "EventManager: 2 registered callbacks"
        );
    }
}
