/// Allows the serialization and deserialization of `Arc<RwLock<T>>` types.
/// It is only available if the `serde` feature is enabled.
pub mod serde_arc_rwlock {
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
        use parking_lot::RwLock;
        use serde::{Deserialize, Serialize};
        use serde_json;
        use std::sync::Arc;

        #[derive(Serialize, Deserialize)]
        struct MyStruct {
            #[serde(with = "crate::utils::serde_arc_rwlock")]
            test: Arc<RwLock<u8>>,
        }
        impl MyStruct {
            pub fn new(test: u8) -> Self {
                Self {
                    test: Arc::new(RwLock::new(test)),
                }
            }
        }

        #[test]
        fn test_serialize() {
            let test = MyStruct::new(20);

            let serialized = serde_json::to_string(&test);
            assert!(serialized.is_ok());

            let expected_json = r#"{"state":20,"default":0,"locked_state":42}"#;
            assert_eq!(serialized.unwrap(), expected_json);
        }

        #[test]
        fn test_deserialize() {
            let json_data = r#"{"state":20,"default":0,"locked_state":42}"#;
            let deserialized = serde_json::from_str::<MyStruct>(json_data);

            assert!(deserialized.is_ok());
            assert_eq!(*deserialized.unwrap().test.read(), 42);
        }
    }
}

/// Allows the serialization and deserialization of `Arc<AtomicPrimitive>` types.
/// It is only available if the `serde` feature is enabled.
pub mod serde_arc_atomic {
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
            #[serde(with = "crate::utils::serde_arc_atomic")]
            a_bool: Arc<AtomicBool>,

            #[serde(with = "crate::utils::serde_arc_atomic")]
            a_u8: Arc<AtomicU8>,

            #[serde(with = "crate::utils::serde_arc_atomic")]
            a_u16: Arc<AtomicU16>,

            #[serde(with = "crate::utils::serde_arc_atomic")]
            a_u32: Arc<AtomicU32>,

            #[serde(with = "crate::utils::serde_arc_atomic")]
            a_u64: Arc<AtomicU64>,

            #[serde(with = "crate::utils::serde_arc_atomic")]
            a_isize: Arc<AtomicIsize>,

            #[serde(with = "crate::utils::serde_arc_atomic")]
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

/// Allows the serialization and deserialization of `Arc<AtomicPrimitive>` types as if `PinMode`.
/// It is only available if the `serde` feature is enabled.
pub mod serde_mode {
    use crate::hardware::PinModeId;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::atomic::*;
    use std::sync::Arc;

    pub fn serialize<S>(mode: &Arc<AtomicU8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        PinModeId::from(mode).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<AtomicU8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mode = PinModeId::deserialize(deserializer)?;
        Ok(Arc::new(AtomicU8::new(mode as u8)))
    }
}

/// Enables serialization and deserialization of `Arc<dyn IoProtocol>` values.
/// Only available when the `serde` feature is enabled.
///
/// # Warning
///
/// `Board` instances are the single source of truth for the protocol instance.
/// Other components (like devices such as `Led`) must share this protocol.
///
/// Therefore, after deserializing a device, you must manually assign the correct protocol:
///
/// ```ignore
/// let board: Board = serde_json::from_str(json_board_data)?;
/// let mut led: Led = serde_json::from_str(json_led_data)?;
/// led.set_protocol(board.get_protocol());
/// ```
pub mod serde_arc_protocol {
    use crate::errors::Error;
    use crate::hardware::{LowLevelApi, Pin, PinModeId};
    use crate::protocols::IoProtocol;
    use crate::utils::Range;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};
    use std::sync::Arc;
    use typetag;

    pub fn serialize<S>(protocol: &Arc<dyn IoProtocol>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Delegate to serde (works because the trait has #[typetag::serde])
        protocol.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<dyn IoProtocol>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let boxed: Box<dyn IoProtocol> = Deserialize::deserialize(deserializer)?;
        Ok(Arc::from(boxed))
    }

    pub fn get_default() -> Arc<dyn IoProtocol> {
        Arc::new(DummySerdeProtocol)
    }

    /// Dummy protocol used during `serde` deserialization of a device.
    ///
    /// Items (e.g. `Led`) depend on the protocol provided by the board. Since only the `Board`
    /// holds the actual, usable protocol instance, deserialized devices must have their protocol
    /// set manually after deserialization.
    ///
    /// ```ignore
    /// let board: Board = serde_json::from_str(json_board_data)?;
    /// let mut led: Led = serde_json::from_str(json_led_data)?;
    /// led.set_protocol(board.get_protocol());
    /// ```
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct DummySerdeProtocol;

    impl DummySerdeProtocol {
        fn throw_error<T>(&self) -> Result<T, Error> {
            Err(Error::InternalError {
                info: "No protocol assigned after deserialization".to_string(),
            })
        }
    }

    impl LowLevelApi for DummySerdeProtocol {
        fn get_protocol_name(&self) -> &str {
            "DummySerdeProtocol"
        }

        fn get_protocol_version(&self) -> &str {
            "N/A"
        }

        fn get_firmware_name(&self) -> &str {
            "DummySerdeFirmware"
        }

        fn get_firmware_version(&self) -> &str {
            "N/A"
        }

        fn get_pins(&self) -> &HashMap<u8, Arc<Pin>> {
            unimplemented!("No protocol assigned after deserialization")
        }

        fn set_pin_mode(&self, _: u8, _: PinModeId) -> Result<(), Error> {
            self.throw_error()
        }

        fn digital_write(&self, _: u8, _: bool) -> Result<(), Error> {
            self.throw_error()
        }

        fn analog_write(&self, _: u8, _: u16) -> Result<(), Error> {
            self.throw_error()
        }

        fn digital_read(&self, _: u8) -> Result<bool, Error> {
            self.throw_error()
        }

        fn analog_read(&self, _: u8) -> Result<u16, Error> {
            self.throw_error()
        }

        fn servo_config(&self, _: u8, _: Range<u16>) -> Result<(), Error> {
            self.throw_error()
        }

        fn i2c_config(&self, _: u16) -> Result<(), Error> {
            self.throw_error()
        }

        fn i2c_read(&self, _: u8, _: u16) -> Result<(), Error> {
            self.throw_error()
        }

        fn i2c_write(&self, _: u8, _: &[u16]) -> Result<(), Error> {
            self.throw_error()
        }
    }

    #[typetag::serde]
    impl IoProtocol for DummySerdeProtocol {
        fn open(&self) -> Result<(), Error> {
            self.throw_error()
        }

        fn close(&self) -> Result<(), Error> {
            self.throw_error()
        }

        fn report_analog(&self, _: u8, _: bool) -> Result<(), Error> {
            self.throw_error()
        }

        fn report_digital(&self, _: u8, _: bool) -> Result<(), Error> {
            self.throw_error()
        }

        fn sampling_interval(&self, _: u16) -> Result<(), Error> {
            self.throw_error()
        }
    }

    impl Display for DummySerdeProtocol {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.get_protocol_name())
        }
    }
}

pub mod serde_arc_transport {
    use crate::transports::IoTransport;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;

    pub fn serialize<S>(protocol: &Arc<dyn IoTransport>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Delegate to serde (works because the trait has #[typetag::serde])
        protocol.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<dyn IoTransport>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let boxed: Box<dyn IoTransport> = Deserialize::deserialize(deserializer)?;
        Ok(Arc::from(boxed))
    }
}

// Helper for serialize skip method.
pub(crate) fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

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
