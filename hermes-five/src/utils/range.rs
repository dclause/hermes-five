#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Range {
    pub min: u16,
    pub max: u16,
}

impl From<[u16; 2]> for Range {
    fn from(value: [u16; 2]) -> Self {
        Self {
            min: value[0],
            max: value[1],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_creation() {
        let range = Range { min: 10, max: 20 };
        assert_eq!(range.min, 10);
        assert_eq!(range.max, 20);
    }

    #[test]
    fn test_range_from_array() {
        let array = [5, 15];
        let range: Range = array.into();
        assert_eq!(range.min, 5);
        assert_eq!(range.max, 15);
    }

    #[test]
    fn test_range_copy_clone() {
        let range1 = Range { min: 2, max: 8 };
        let range2 = range1; // Copy
        let range3 = range1.clone(); // Clone

        assert_eq!(range1.min, range2.min);
        assert_eq!(range1.max, range2.max);
        assert_eq!(range1.min, range3.min);
        assert_eq!(range1.max, range3.max);
    }

    #[test]
    fn test_range_equality() {
        let range1 = Range { min: 3, max: 9 };
        let range2 = Range { min: 3, max: 9 };
        assert_eq!(range1, range2);
        let range3 = Range { min: 3, max: 10 };
        assert_ne!(range1, range3);
        let range4 = Range { min: 4, max: 10 };
        assert_ne!(range1, range4);
    }

    #[test]
    fn test_range_debug() {
        let range = Range { min: 4, max: 10 };
        let debug_str = format!("{:?}", range);
        assert_eq!(debug_str, "Range { min: 4, max: 10 }");
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use serde_json;

        use super::*;

        #[test]
        fn test_range_serialize() {
            let range = Range { min: 6, max: 12 };
            let json = serde_json::to_string(&range).unwrap();
            assert_eq!(json, r#"{"min":6,"max":12}"#);
        }

        #[test]
        fn test_range_deserialize() {
            let json = r#"{"min":7,"max":14}"#;
            let range: Range = serde_json::from_str(json).unwrap();
            assert_eq!(range.min, 7);
            assert_eq!(range.max, 14);
        }
    }
}
