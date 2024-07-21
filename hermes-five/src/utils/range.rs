#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Range<T> {
    pub start: T,
    pub end: T,
}

impl<T: Copy> From<[T; 2]> for Range<T> {
    fn from(value: [T; 2]) -> Self {
        Self {
            start: value[0],
            end: value[1],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_creation() {
        let range = Range { start: 10, end: 20 };
        assert_eq!(range.start, 10);
        assert_eq!(range.end, 20);
    }

    #[test]
    fn test_range_from_array() {
        let array = [5, 15];
        let range: Range<u8> = array.into();
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 15);
    }

    #[test]
    fn test_range_copy_clone() {
        let range1 = Range { start: 2, end: 8 };
        let range2 = range1; // Copy
        let range3 = range1.clone(); // Clone

        assert_eq!(range1.start, range2.start);
        assert_eq!(range1.end, range2.end);
        assert_eq!(range1.start, range3.start);
        assert_eq!(range1.end, range3.end);
    }

    #[test]
    fn test_range_equality() {
        let range1 = Range { start: 3, end: 9 };
        let range2 = Range { start: 3, end: 9 };
        assert_eq!(range1, range2);
        let range3 = Range { start: 3, end: 10 };
        assert_ne!(range1, range3);
        let range4 = Range { start: 4, end: 10 };
        assert_ne!(range1, range4);
    }

    #[test]
    fn test_range_debug() {
        let range = Range { start: 4, end: 10 };
        let debug_str = format!("{:?}", range);
        assert_eq!(debug_str, "Range { min: 4, max: 10 }");
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use serde_json;

        use super::*;

        #[test]
        fn test_range_serialize() {
            let range = Range { start: 6, end: 12 };
            let json = serde_json::to_string(&range).unwrap();
            assert_eq!(json, r#"{"min":6,"max":12}"#);
        }

        #[test]
        fn test_range_deserialize() {
            let json = r#"{"min":7,"max":14}"#;
            let range: Range = serde_json::from_str(json).unwrap();
            assert_eq!(range.start, 7);
            assert_eq!(range.end, 14);
        }
    }
}
