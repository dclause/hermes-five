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

#[cfg(feature = "serde")]
impl<T> serde::Serialize for Range<T>
where
    T: serde::Serialize + Copy,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize the Range as an array [start, end]
        [self.start, self.end].serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Range<T>
where
    T: serde::Deserialize<'de> + Copy,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize from an array [start, end]
        let array: [T; 2] = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self::from(array))
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
        #[allow(clippy::clone_on_copy)]
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
        assert_eq!(debug_str, "Range { start: 4, end: 10 }");
    }

    #[cfg(feature = "serde")]
    #[cfg(test)]
    mod serde_tests {
        use serde_json;

        use super::*;

        #[test]
        fn test_range_serialize() {
            let range = Range { start: 6, end: 12 };
            let json = serde_json::to_string(&range).unwrap();
            assert_eq!(json, r#"[6,12]"#);
        }

        #[test]
        fn test_range_deserialize() {
            let json = r#"[7,14]"#;
            let range: Range<u8> = serde_json::from_str(json).unwrap();
            assert_eq!(range.start, 7);
            assert_eq!(range.end, 14);
        }
    }
}
