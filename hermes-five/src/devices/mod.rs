use dyn_clone::DynClone;
use std::fmt::{Debug, Display};

mod input;
mod output;

// Input devices re-exports
pub use crate::devices::input::analog::AnalogInput;
pub use crate::devices::input::button::Button;
pub use crate::devices::input::digital::DigitalInput;
pub use crate::devices::input::{Input, InputEvent};
// Output devices re-exports
pub use crate::devices::output::digital::DigitalOutput;
pub use crate::devices::output::led::Led;
pub use crate::devices::output::servo::Servo;
pub use crate::devices::output::servo::ServoType;
pub use crate::devices::output::Output;

/// A trait for devices that can be debugged, cloned, and used in concurrent contexts.
/// [`Device`] are one of the `Entity` defined in Hermes-Five project: it represents a physical
/// device that is plugged to and can be controlled by a [`Board`]. `Device`s come in two flavor:
/// - `Actuator`: device that can act on the world
/// - `Sensor`: device that can sense or measure data from the world
///
/// Implementors of this trait are required to be `Debug`, `DynClone`, `Send`, and `Sync`.
/// This ensures that devices can be cloned and used safely in multithreaded and async environments.
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Device: Debug + Display + DynClone + Send + Sync {}
dyn_clone::clone_trait_object!(Device);

#[cfg(feature = "serde")]
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
    mod arc_rwlock_serde_tests {
        use serde_json;

        use crate::mocks::output::MockOutputDevice;

        #[test]
        fn test_serialize() {
            let test = MockOutputDevice::new(20);

            let serialized = serde_json::to_string(&test);
            assert!(serialized.is_ok());

            let expected_json = r#"{"state":20,"lock":42}"#;
            assert_eq!(serialized.unwrap(), expected_json);
        }

        #[test]
        fn test_deserialize() {
            let json_data = r#"{"state":20,"lock":42}"#;
            let deserialized = serde_json::from_str::<MockOutputDevice>(json_data);

            assert!(deserialized.is_ok());
            assert_eq!(deserialized.unwrap().get_locked_value(), 42);
        }
    }
}
