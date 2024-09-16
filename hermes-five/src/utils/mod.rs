pub use log;
#[cfg(test)]
pub use serial_test;
pub use tokio;

pub use crate::utils::easing::Easing;
pub use crate::utils::range::Range;
pub use crate::utils::state::State;

mod easing;
mod range;
mod state;

pub mod events;
pub mod scale;
pub mod task;

#[cfg(feature = "serde")]
// Helper for serialize skip method.
pub(crate) fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod tests {
    use crate::utils::is_default;
    use crate::utils::State::Boolean;

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
