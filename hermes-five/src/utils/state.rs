use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum State {
    #[default]
    Null,
    Boolean(bool),
    Integer(u64),
    Signed(i64),
    Float(f64),
    String(String),
    Array(Vec<State>),
    Object(HashMap<String, State>),
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Null => write!(f, "Null"),
            State::Boolean(b) => write!(f, "{}", b),
            State::Integer(i) => write!(f, "{}", i),
            State::Signed(s) => write!(f, "{}", s),
            State::Float(fl) => write!(f, "{}", fl),
            State::String(s) => write!(f, "\"{}\"", s),
            State::Array(arr) => {
                let elements = arr
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "[{}]", elements)
            }
            State::Object(obj) => {
                let entries = obj
                    .iter()
                    .map(|(key, value)| format!("\"{}\": {}", key, value))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{{{}}}", entries)
            }
        }
    }
}

// **********************************************
// Serde
// **********************************************

#[cfg(feature = "serde")]
impl serde::Serialize for State {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            State::Null => serializer.serialize_none(),
            State::Boolean(b) => serializer.serialize_bool(*b),
            State::Integer(i) => serializer.serialize_u64(*i),
            State::Signed(i) => serializer.serialize_i64(*i),
            State::Float(f) => serializer.serialize_f64(*f),
            State::String(s) => serializer.serialize_str(s),
            State::Array(a) => serializer.collect_seq(a),
            State::Object(o) => serializer.collect_map(o),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> ::serde::Deserialize<'de> for State {
    fn deserialize<D>(de: D) -> Result<State, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(State::from(serde_json::Value::deserialize(de)?))
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Value> for State {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => State::Null,
            serde_json::Value::Bool(b) => State::Boolean(b),
            serde_json::Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    State::Integer(u)
                } else if let Some(i) = n.as_i64() {
                    State::Signed(i)
                } else {
                    State::Float(n.as_f64().unwrap())
                }
            }
            serde_json::Value::String(s) => State::String(s),
            serde_json::Value::Array(list) => {
                State::Array(list.into_iter().map(Into::into).collect())
            }
            serde_json::Value::Object(map) => State::Object(
                map.into_iter()
                    .map(|(key, value)| (key, value.into()))
                    .collect(),
            ),
        }
    }
}

#[cfg(feature = "serde")]
impl State {
    pub fn into_state<T: serde::Serialize>(value: T) -> State {
        serde_json::to_value(value).unwrap().into()
    }
}

// **********************************************
// Extractors: get the value inside State.
// **********************************************
impl State {
    pub fn is_null(&self) -> bool {
        *self == State::Null
    }

    /// Extracts the boolean value if it is a boolean.
    pub fn as_bool(&self) -> bool {
        match self {
            State::Null => false,
            State::Boolean(b) => *b,
            State::Integer(u) => *u > 0,
            State::Signed(i) => *i > 0,
            State::Float(f) => *f > 0.0,
            State::String(s) => !s.is_empty(),
            State::Array(a) => !a.is_empty(),
            State::Object(o) => !o.is_empty(),
        }
    }

    /// Extracts the integer value if it is an integer.
    pub fn as_integer(&self) -> u64 {
        match *self {
            State::Boolean(b) => u64::from(b),
            State::Integer(u) => u,
            State::Signed(i) => match i > 0 {
                true => i as u64,
                false => 0,
            },
            State::Float(f) => f as u64,
            _ => 0,
        }
    }
    /// Extracts the signed integer value if it is an integer.
    pub fn as_signed_integer(&self) -> i64 {
        match *self {
            State::Boolean(b) => i64::from(b),
            State::Integer(i) => i as i64,
            State::Signed(i) => i,
            State::Float(f) => f as i64,
            _ => 0,
        }
    }
    /// Extracts the float value if it is a float.
    pub fn as_float(&self) -> f64 {
        match *self {
            State::Boolean(b) => f64::from(b),
            State::Integer(u) => u as f64,
            State::Signed(i) => i as f64,
            State::Float(f) => f,
            _ => 0.0,
        }
    }

    /// Extracts the string of this value if it is a string.
    pub fn as_string(&self) -> String {
        match self {
            State::Integer(u) => format!("{}", u),
            State::Signed(i) => format!("{}", i),
            State::Float(f) => format!("{}", f),
            State::String(s) => s.clone(),
            _ => String::default(),
        }
    }

    /// Extracts the &str of this value if it is a string.
    pub fn as_str(&self) -> &str {
        match self {
            State::String(ref s) => s,
            _ => "",
        }
    }
    /// Extracts the array value if it is an array.
    pub fn as_array(&self) -> Vec<State> {
        match *self {
            State::Array(ref a) => a.clone(),
            _ => vec![],
        }
    }
    /// Extracts the hashmap value if it is a hashmap.
    pub fn as_object(&self) -> HashMap<String, State> {
        match self {
            State::Object(map) => map.clone(),
            _ => HashMap::<String, State>::default(),
        }
    }
}

// **********************************************
// Converters: set a value inside State.
// **********************************************

macro_rules! impl_from_converter {
    ($variant:ident : $T:ty) => {
        impl From<$T> for State {
            #[inline]
            fn from(val: $T) -> State {
                State::$variant(val.into())
            }
        }
    };
}

impl_from_converter!(String: String);
impl_from_converter!(Integer: u8);
impl_from_converter!(Integer: u16);
impl_from_converter!(Integer: u32);
impl_from_converter!(Integer: u64);
impl_from_converter!(Signed: i8);
impl_from_converter!(Signed: i16);
impl_from_converter!(Signed: i32);
impl_from_converter!(Signed: i64);
impl_from_converter!(Float: f32);
impl_from_converter!(Float: f64);
impl_from_converter!(Boolean: bool);
// impl_from_converter!(Array: Vec<State>);
// impl_from_converter!(Object: HashMap<String, State>);

impl<T: Into<State>> From<Vec<T>> for State {
    /// Convert a `Vec` to `State::Array`.
    fn from(f: Vec<T>) -> Self {
        State::Array(f.into_iter().map(Into::into).collect())
    }
}

impl<T: Clone + Into<State>> From<&[T]> for State {
    /// Convert a slice to `State::Array`.
    fn from(f: &[T]) -> Self {
        State::Array(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<State>> FromIterator<T> for State {
    /// Create a `State::Array` by collecting an iterator of array elements.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        State::Array(iter.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_as_boolean() {
        assert!(!State::Null.as_bool());

        assert!(!State::Boolean(false).as_bool());
        assert!(State::Boolean(true).as_bool());

        assert!(!State::Integer(0).as_bool());
        assert!(State::Integer(10).as_bool());

        assert!(!State::Signed(-10).as_bool());
        assert!(!State::Signed(0).as_bool());
        assert!(State::Signed(10).as_bool());

        assert!(!State::Float(-0.5).as_bool());
        assert!(!State::Float(0.0).as_bool());
        assert!(State::Float(10.5).as_bool());

        assert!(!State::String(String::from("")).as_bool());
        assert!(State::String(" ".into()).as_bool());

        assert!(!State::Array(vec!()).as_bool());
        assert!(State::Array(vec![1.into()]).as_bool());

        let mut map = HashMap::new();
        assert!(!State::Object(map.clone()).as_bool());
        map.insert("key".to_string(), State::Integer(42));
        assert!(State::Object(map).as_bool());
    }

    #[test]
    fn test_as_integer() {
        assert_eq!(State::Null.as_integer(), 0);
        assert_eq!(State::Boolean(false).as_integer(), 0);
        assert_eq!(State::Boolean(true).as_integer(), 1);
        assert_eq!(State::Integer(42).as_integer(), 42);
        assert_eq!(State::Signed(-12).as_integer(), 0);
        assert_eq!(State::Signed(12).as_integer(), 12);
        assert_eq!(State::Float(69.5).as_integer(), 69);
        assert_eq!(State::Float(-69.5).as_integer(), 0);
        assert_eq!(State::String(String::from("test")).as_integer(), 0);
        assert_eq!(State::Array(vec![1.into()]).as_integer(), 0);
        assert_eq!(State::Object(HashMap::new()).as_integer(), 0);
    }

    #[test]
    fn test_as_signed_integer() {
        assert_eq!(State::Null.as_signed_integer(), 0);
        assert_eq!(State::Boolean(false).as_signed_integer(), 0);
        assert_eq!(State::Boolean(true).as_signed_integer(), 1);
        assert_eq!(State::Integer(42).as_signed_integer(), 42);
        assert_eq!(State::Signed(-12).as_signed_integer(), -12);
        assert_eq!(State::Signed(12).as_signed_integer(), 12);
        assert_eq!(State::Float(69.5).as_signed_integer(), 69);
        assert_eq!(State::Float(-69.5).as_signed_integer(), -69);
        assert_eq!(State::String(String::from("test")).as_signed_integer(), 0);
        assert_eq!(State::Array(vec![1.into()]).as_signed_integer(), 0);
        assert_eq!(State::Object(HashMap::new()).as_signed_integer(), 0);
    }

    #[test]
    fn test_as_float() {
        assert_eq!(State::Null.as_float(), 0.0);
        assert_eq!(State::Boolean(false).as_float(), 0.0);
        assert_eq!(State::Boolean(true).as_float(), 1.0);
        assert_eq!(State::Integer(42).as_float(), 42.0);
        assert_eq!(State::Signed(-12).as_float(), -12.0);
        assert_eq!(State::Signed(12).as_float(), 12.0);
        assert_eq!(State::Float(69.5).as_float(), 69.5);
        assert_eq!(State::Float(-69.5).as_float(), -69.5);
        assert_eq!(State::String(String::from("test")).as_float(), 0.0);
        assert_eq!(State::Array(vec![1.into()]).as_float(), 0.0);
        assert_eq!(State::Object(HashMap::new()).as_float(), 0.0);
    }

    #[test]
    fn test_as_string() {
        assert_eq!(State::Null.as_string(), String::from(""));
        assert_eq!(State::Boolean(false).as_string(), String::from(""));
        assert_eq!(State::Boolean(true).as_string(), String::from(""));
        assert_eq!(State::Integer(42).as_string(), String::from("42"));
        assert_eq!(State::Signed(-12).as_string(), String::from("-12"));
        assert_eq!(State::Signed(12).as_string(), String::from("12"));
        assert_eq!(State::Float(69.5).as_string(), String::from("69.5"));
        assert_eq!(
            State::String(String::from("test")).as_string(),
            String::from("test")
        );
        assert_eq!(State::Array(vec![1.into()]).as_string(), String::from(""));
        assert_eq!(State::Object(HashMap::new()).as_string(), String::from(""));
    }

    #[test]
    fn test_as_str() {
        assert_eq!(State::Null.as_str(), "");
        assert_eq!(State::Boolean(false).as_str(), "");
        assert_eq!(State::Boolean(true).as_str(), "");
        assert_eq!(State::Integer(42).as_str(), "");
        assert_eq!(State::Signed(-12).as_str(), "");
        assert_eq!(State::Signed(12).as_str(), "");
        assert_eq!(State::Float(69.5).as_str(), "");
        assert_eq!(State::String(String::from("test")).as_str(), "test");
        assert_eq!(State::Array(vec![1.into()]).as_str(), "");
        assert_eq!(State::Object(HashMap::new()).as_str(), "");
    }

    #[test]
    fn test_as_array() {
        assert_eq!(State::Null.as_array(), vec![]);
        assert_eq!(State::Boolean(false).as_array(), vec![]);
        assert_eq!(State::Boolean(true).as_array(), vec![]);
        assert_eq!(State::Integer(42).as_array(), vec![]);
        assert_eq!(State::Signed(-12).as_array(), vec![]);
        assert_eq!(State::Signed(12).as_array(), vec![]);
        assert_eq!(State::Float(69.5).as_array(), vec![]);
        assert_eq!(State::String(String::from("test")).as_array(), vec![]);
        assert_eq!(
            State::Array(vec![1u8.into(), 2u8.into()]).as_array(),
            vec![State::Integer(1), State::Integer(2)]
        );
        assert_eq!(State::Object(HashMap::new()).as_array(), vec![]);
    }

    #[test]
    fn test_as_object() {
        let empty = HashMap::new();
        assert_eq!(State::Null.as_object(), empty);
        assert_eq!(State::Boolean(false).as_object(), empty);
        assert_eq!(State::Boolean(true).as_object(), empty);
        assert_eq!(State::Integer(42).as_object(), empty);
        assert_eq!(State::Signed(-12).as_object(), empty);
        assert_eq!(State::Signed(12).as_object(), empty);
        assert_eq!(State::Float(69.5).as_object(), empty);
        assert_eq!(State::String(String::from("test")).as_object(), empty);
        assert_eq!(State::Array(vec![1.into()]).as_object(), empty);

        let mut map = HashMap::new();
        map.insert("key".to_string(), State::Integer(42));

        let state = State::Object(map.clone());
        assert_eq!(state.as_object(), map);
    }

    #[test]
    fn test_is_null() {
        assert!(State::Null.is_null());
        assert!(!State::Boolean(false).is_null());
        assert!(!State::Boolean(true).is_null());
        assert!(!State::Integer(42).is_null());
        assert!(!State::Signed(-12).is_null());
        assert!(!State::Signed(12).is_null());
        assert!(!State::Float(69.5).is_null());
        assert!(!State::String(String::from("test")).is_null());
        assert!(!State::Array(vec![1.into()]).is_null());
        assert!(!State::Object(HashMap::new()).is_null());
    }

    #[test]
    fn test_from_conversions() {
        let state: State = "test".to_string().into();
        assert_eq!(state, State::String("test".into()));

        let state: State = 42u8.into();
        assert_eq!(state, State::Integer(42));

        let state: State = 42u16.into();
        assert_eq!(state, State::Integer(42));

        let state: State = 42u32.into();
        assert_eq!(state, State::Integer(42));

        let state: State = 42u64.into();
        assert_eq!(state, State::Integer(42));

        let state: State = (-42i8).into();
        assert_eq!(state, State::Signed(-42));

        let state: State = (-42i16).into();
        assert_eq!(state, State::Signed(-42));

        let state: State = (-42i32).into();
        assert_eq!(state, State::Signed(-42));

        let state: State = (-42i64).into();
        assert_eq!(state, State::Signed(-42));

        let state: State = std::f32::consts::PI.into();
        assert!(matches!(state, State::Float(f) if (f - std::f64::consts::PI).abs() < 0.00001),);

        let state: State = std::f64::consts::PI.into();
        assert_eq!(state, State::Float(std::f64::consts::PI));

        let state: State = true.into();
        assert_eq!(state, State::Boolean(true));

        let vec_state: State = vec![1u8, 2u8, 3u8].into();
        assert_eq!(
            vec_state,
            State::Array(vec![
                State::Integer(1),
                State::Integer(2),
                State::Integer(3)
            ])
        );

        let state: State = [1, 2].as_slice().into();
        assert_eq!(
            state,
            State::Array(vec![State::Signed(1), State::Signed(2)])
        );

        let state: State = vec![1, 2].into_iter().collect();
        assert_eq!(
            state,
            State::Array(vec![State::Signed(1), State::Signed(2)])
        );
    }

    #[test]
    fn test_display_null() {
        let state = State::Null;
        assert_eq!(state.to_string(), "Null");
    }

    #[test]
    fn test_display_boolean() {
        let state_true = State::Boolean(true);
        let state_false = State::Boolean(false);
        assert_eq!(state_true.to_string(), "true");
        assert_eq!(state_false.to_string(), "false");
    }

    #[test]
    fn test_display_integer() {
        let state = State::Integer(42);
        assert_eq!(state.to_string(), "42");
    }

    #[test]
    fn test_display_signed() {
        let state_positive = State::Signed(42);
        let state_negative = State::Signed(-42);
        assert_eq!(state_positive.to_string(), "42");
        assert_eq!(state_negative.to_string(), "-42");
    }

    #[test]
    fn test_display_float() {
        #[allow(clippy::approx_constant)]
        let state = State::Float(3.14);
        assert_eq!(state.to_string(), "3.14");
    }

    #[test]
    fn test_display_string() {
        let state = State::String("Hello".to_string());
        assert_eq!(state.to_string(), "\"Hello\"");
    }

    #[test]
    fn test_display_array() {
        let state = State::Array(vec![
            State::Integer(1),
            State::Boolean(false),
            State::String("test".to_string()),
        ]);
        assert_eq!(state.to_string(), "[1, false, \"test\"]");
    }

    #[test]
    fn test_display_object() {
        let mut obj = HashMap::new();
        obj.insert("key1".to_string(), State::Integer(10));
        obj.insert("key2".to_string(), State::Boolean(true));

        let state = State::Object(obj);
        // Since HashMap does not guarantee ordering, we'll check both possible orderings
        let result = state.to_string();
        let expected1 = "{\"key1\": 10, \"key2\": true}";
        let expected2 = "{\"key2\": true, \"key1\": 10}";

        assert!(result == expected1 || result == expected2);
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use serde_json;

        use super::*;

        #[test]
        fn test_serialize_null() {
            let state = State::Null;
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, "null");
        }

        #[test]
        fn test_serialize_boolean() {
            let state = State::Boolean(true);
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, "true");
        }

        #[test]
        fn test_serialize_integer() {
            let state = State::Integer(42);
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, "42");
        }

        #[test]
        fn test_serialize_signed() {
            let state = State::Signed(-42);
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, "-42");
        }

        #[test]
        fn test_serialize_float() {
            let state = State::Float(3.14);
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, "3.14");
        }

        #[test]
        fn test_serialize_string() {
            let state = State::String("test".into());
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, r#""test""#);
        }

        #[test]
        fn test_serialize_array() {
            let state = State::Array(vec![State::Integer(1), State::Integer(2)]);
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, "[1,2]");
        }

        #[test]
        fn test_serialize_object() {
            let mut map = HashMap::new();
            map.insert("key".to_string(), State::Integer(42));
            let state = State::Object(map);
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, r#"{"key":42}"#);
        }

        #[test]
        fn test_deserialize_null() {
            let json = "null";
            let state: State = serde_json::from_str(json).unwrap();
            assert_eq!(state, State::Null);
        }

        #[test]
        fn test_deserialize_boolean() {
            let json = "true";
            let state: State = serde_json::from_str(json).unwrap();
            assert_eq!(state, State::Boolean(true));
        }

        #[test]
        fn test_deserialize_integer() {
            let json = "42";
            let state: State = serde_json::from_str(json).unwrap();
            assert_eq!(state, State::Integer(42));
        }

        #[test]
        fn test_deserialize_signed() {
            let json = "-42";
            let state: State = serde_json::from_str(json).unwrap();
            assert_eq!(state, State::Signed(-42));
        }

        #[test]
        fn test_deserialize_float() {
            let json = "3.14";
            let state: State = serde_json::from_str(json).unwrap();
            assert_eq!(state, State::Float(3.14));
        }

        #[test]
        fn test_deserialize_nan() {
            let json = serde_json::to_string(&(f64::NAN)).unwrap();
            let state: State = serde_json::from_str(json.as_str()).unwrap();
            assert_eq!(state, State::Null);
        }

        #[test]
        fn test_deserialize_string() {
            let json = r#""test""#;
            let state: State = serde_json::from_str(json).unwrap();
            assert_eq!(state, State::String("test".into()));
        }

        #[test]
        fn test_deserialize_array() {
            let json = "[1,2]";
            let state: State = serde_json::from_str(json).unwrap();
            assert_eq!(
                state,
                State::Array(vec![State::Integer(1), State::Integer(2)])
            );
        }

        #[test]
        fn test_deserialize_object() {
            let json = r#"{"key":42}"#;
            let state: State = serde_json::from_str(json).unwrap();
            let mut map = HashMap::new();
            map.insert("key".to_string(), State::Integer(42));
            assert_eq!(state, State::Object(map));
        }

        #[test]
        fn test_deserialize_complex() {
            let json = r#"{"key":42, "state": {"key": 42}}"#;
            let state: State = serde_json::from_str(json).unwrap();
            let mut map = HashMap::new();
            map.insert("key".to_string(), State::Integer(42));
            map.insert("state".to_string(), State::Object(map.clone()));
            assert_eq!(state, State::Object(map));
        }

        #[test]
        fn test_into_state() {
            let input = "hello world";
            let state = State::into_state(input);
            assert_eq!(state.as_string(), String::from("hello world"));
        }
    }
}
