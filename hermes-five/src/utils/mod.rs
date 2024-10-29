//! Various utilities and helper functions.

pub use log;
#[cfg(test)]
pub use serial_test;
pub use tokio;

mod events;
mod range;
mod scale;
mod state;
pub mod task;

pub use crate::utils::events::*;
pub use crate::utils::range::*;
pub use crate::utils::scale::*;
pub use crate::utils::state::*;
pub use crate::utils::task::*;

#[cfg(feature = "serde")]
// Helper for serialize skip method.
pub(crate) fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod tests {
    use crate::utils::is_default;

    #[test]
    fn test_is_default() {
        // Bool
        assert_eq!(is_default(&true), false);
        assert_eq!(is_default(&false), true);
        // String
        assert_eq!(is_default(&String::new()), true);
        assert_eq!(is_default(&String::from("test")), false);
        // usize
        assert_eq!(is_default(&0), true);
        assert_eq!(is_default(&69), false);

        // ....
    }
}
