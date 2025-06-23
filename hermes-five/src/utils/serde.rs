/// Allows the serialization and deserialization of `Arc<RwLock<T>>` types.
/// It is only available if the `serde` feature is enabled.
pub mod arc_rwlock_serde {
    use std::sync::Arc;

    use parking_lot::RwLock;
    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S, T>(val: &Arc<RwLock<T>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        T::serialize(&*val.read(), s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Arc<RwLock<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Ok(Arc::new(RwLock::new(T::deserialize(d)?)))
    }

    #[cfg(test)]
    mod serde_tests {
        use serde_json;

        use crate::mocks::MockOutputDevice;

        #[test]
        fn test_serialize() {
            let test = MockOutputDevice::new(20);

            let serialized = serde_json::to_string(&test);
            assert!(serialized.is_ok());

            let expected_json = r#"{"state":20,"locked_state":42}"#;
            assert_eq!(serialized.unwrap(), expected_json);
        }

        #[test]
        fn test_deserialize() {
            let json_data = r#"{"state":20,"locked_state":42}"#;
            let deserialized = serde_json::from_str::<MockOutputDevice>(json_data);

            assert!(deserialized.is_ok());
            assert_eq!(deserialized.unwrap().get_locked_value(), 42);
        }
    }
}

pub mod arc_atomic_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::atomic::*;
    use std::sync::Arc;

    // Trait that abstracts atomic load/store with a primitive type
    pub trait AtomicPrimitive: Sized {
        type Primitive: Copy + Serialize + for<'de> Deserialize<'de>;

        fn load(&self, order: Ordering) -> Self::Primitive;
        fn new(v: Self::Primitive) -> Self;
    }

    // Implement it for each atomic type

    macro_rules! impl_atomic_primitive {
        ($atomic:ty, $prim:ty) => {
            impl AtomicPrimitive for $atomic {
                type Primitive = $prim;

                fn load(&self, order: Ordering) -> Self::Primitive {
                    self.load(order)
                }

                fn new(v: Self::Primitive) -> Self {
                    <$atomic>::new(v)
                }
            }
        };
    }

    impl_atomic_primitive!(AtomicBool, bool);
    impl_atomic_primitive!(AtomicU8, u8);
    impl_atomic_primitive!(AtomicU16, u16);
    impl_atomic_primitive!(AtomicU32, u32);
    impl_atomic_primitive!(AtomicU64, u64);
    impl_atomic_primitive!(AtomicIsize, isize);
    impl_atomic_primitive!(AtomicUsize, usize);

    pub fn serialize<S, T>(val: &Arc<T>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AtomicPrimitive,
        T::Primitive: Serialize,
    {
        val.load(Ordering::SeqCst).serialize(s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Arc<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: AtomicPrimitive + Deserialize<'de>,
        T::Primitive: Serialize,
    {
        let val = T::Primitive::deserialize(d)?;
        Ok(Arc::new(T::new(val)))
    }

    #[cfg(test)]
    mod serde_tests {
        use serde::{Deserialize, Serialize};
        use serde_json;
        use std::sync::atomic::*;
        use std::sync::Arc;

        #[derive(Serialize, Deserialize, Debug)]
        struct TestStruct {
            #[serde(with = "crate::utils::arc_atomic_serde")]
            a_bool: Arc<AtomicBool>,

            #[serde(with = "crate::utils::arc_atomic_serde")]
            a_u8: Arc<AtomicU8>,

            #[serde(with = "crate::utils::arc_atomic_serde")]
            a_u16: Arc<AtomicU16>,

            #[serde(with = "crate::utils::arc_atomic_serde")]
            a_u32: Arc<AtomicU32>,

            #[serde(with = "crate::utils::arc_atomic_serde")]
            a_u64: Arc<AtomicU64>,

            #[serde(with = "crate::utils::arc_atomic_serde")]
            a_isize: Arc<AtomicIsize>,

            #[serde(with = "crate::utils::arc_atomic_serde")]
            a_usize: Arc<AtomicUsize>,
        }

        #[test]
        fn test_all_atomic_types_serialization_rountrip() {
            let original = TestStruct {
                a_bool: Arc::new(AtomicBool::new(true)),
                a_u8: Arc::new(AtomicU8::new(8)),
                a_u16: Arc::new(AtomicU16::new(16000)),
                a_u32: Arc::new(AtomicU32::new(320000)),
                a_u64: Arc::new(AtomicU64::new(64_000_000)),
                a_isize: Arc::new(AtomicIsize::new(-42)),
                a_usize: Arc::new(AtomicUsize::new(42)),
            };

            // Serialize to JSON
            let json = serde_json::to_string(&original).unwrap();
            println!("Serialized JSON: {}", json);

            // Deserialize from JSON
            let deserialized: TestStruct = serde_json::from_str(&json).unwrap();

            // Assert equality of all fields
            assert_eq!(deserialized.a_bool.load(Ordering::SeqCst), true);
            assert_eq!(deserialized.a_u8.load(Ordering::SeqCst), 8);
            assert_eq!(deserialized.a_u16.load(Ordering::SeqCst), 16000);
            assert_eq!(deserialized.a_u32.load(Ordering::SeqCst), 320000);
            assert_eq!(deserialized.a_u64.load(Ordering::SeqCst), 64_000_000);
            assert_eq!(deserialized.a_isize.load(Ordering::SeqCst), -42);
            assert_eq!(deserialized.a_usize.load(Ordering::SeqCst), 42);
        }
    }
}
