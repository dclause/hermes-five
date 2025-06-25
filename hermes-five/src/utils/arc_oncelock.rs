use crate::errors::{Error, InternalError};
use std::fmt::Debug;
use std::sync::OnceLock;

/// A trait to set the value of a `OnceLock<T>` wrapped in an `Arc`,
/// returning a contextual error if the value was already set.
///
/// Useful for reducing repetition when initializing shared, lazily-set fields.
///
/// # Example
/// ```exclude
/// arc_lock.set_with_context(value, "Failed to set field")?;
/// ```
pub trait ArcOnceLockExt<T> {
    fn set_with_context(&self, value: T, context: &str) -> Result<(), Error>;
}

impl<T: Debug> ArcOnceLockExt<T> for OnceLock<T> {
    fn set_with_context(&self, value: T, context: &str) -> Result<(), Error> {
        self.set(value).map_err(|e| {
            Error::from(InternalError {
                info: format!("{} cannot be rewrote {:?}", context, e),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, OnceLock};

    #[derive(Debug)]
    struct Dummy;

    fn make_error_string(e: &Error) -> String {
        // Assuming Error::from(InternalError { info }) maps to something retrievable
        format!("{}", e)
    }

    #[test]
    fn test_set_with_context_success() {
        let lock = Arc::new(OnceLock::new());

        let result = lock.set_with_context(Dummy, "setting Dummy");
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_with_context_failure_already_set() {
        let lock = Arc::new(OnceLock::new());

        // First set succeeds
        assert!(lock.set_with_context(Dummy, "initial").is_ok());

        // Second set fails
        let result = lock.set_with_context(Dummy, "already set");
        assert!(result.is_err());

        let err_msg = make_error_string(&result.unwrap_err());
        assert!(err_msg.contains("already set") || err_msg.contains("already initialized"));
        assert!(err_msg.contains("already set")); // Context string must be present
    }

    #[test]
    fn test_plain_once_lock_support() {
        let lock = OnceLock::new();
        let result = lock.set_with_context(42, "plain lock test");
        assert!(result.is_ok());

        let result2 = lock.set_with_context(100, "second attempt");
        assert!(result2.is_err());

        let err_msg = make_error_string(&result2.unwrap_err());
        assert!(err_msg.contains("second attempt"));
    }
}
