//! Various utilities and helper functions.

pub use log;
#[cfg(test)]
pub use serial_test;
use std::fmt::Write;
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

/// Helper to format a buffer as hex.
#[allow(dead_code)]
pub(crate) fn format_as_hex<T: std::fmt::UpperHex>(slice: &[T]) -> String {
    let mut result = String::with_capacity(slice.len() * 5); // Preallocate memory

    for (i, byte) in slice.iter().enumerate() {
        if i > 0 {
            result.push_str(", ");
        }
        // Use `write!` directly to append formatted byte to result
        let _ = write!(result, "0x{:02X}", byte);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::format_as_hex;

    #[test]
    fn test_format_as_hex_empty_slice() {
        let input: [u8; 0] = [];
        let result = format_as_hex(&input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_as_hex_single_element() {
        let input = [15u8];
        let result = format_as_hex(&input);
        assert_eq!(result, "0x0F");
    }

    #[test]
    fn test_format_as_hex_multiple_elements() {
        let input = [15u8, 255, 0, 128];
        let result = format_as_hex(&input);
        assert_eq!(result, "0x0F, 0xFF, 0x00, 0x80");
    }

    #[test]
    fn test_format_as_hex_u16() {
        let input = [4095u16, 65535];
        let result = format_as_hex(&input);
        assert_eq!(result, "0xFFF, 0xFFFF");
    }

    #[test]
    fn test_format_as_hex_large_numbers() {
        let input = [123456789u32, 4294967295];
        let result = format_as_hex(&input);
        assert_eq!(result, "0x75BCD15, 0xFFFFFFFF");
    }
}

#[cfg(feature = "serde")]
// Helper for serialize skip method.
pub(crate) fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod serde_tests {
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
