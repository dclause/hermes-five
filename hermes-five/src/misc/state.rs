use std::collections::HashMap;

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

// **********************************************
// Serde
// **********************************************
#[cfg(feature = "serde")]
impl State {}

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
        let state = serde_json::Value::deserialize(de)?;
        Ok(State::from(state))
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
                } else if let Some(f) = n.as_f64() {
                    State::Float(f)
                } else {
                    State::Integer(0)
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

// **********************************************
// Extractors: get the value inside State.
// **********************************************
impl State {
    pub fn is_null(&self) -> bool {
        *self == State::Null
    }

    /// Extracts the boolean value if it is a boolean.
    pub fn as_boolean(&self) -> Option<bool> {
        match *self {
            State::Null => Some(false),
            State::Boolean(b) => Some(b),
            _ => None,
        }
    }
    /// Extracts the integer value if it is an integer.
    pub fn as_integer(&self) -> Option<u64> {
        match *self {
            State::Integer(i) => Some(i),
            State::Signed(i) => Some(i as u64),
            _ => None,
        }
    }
    /// Extracts the signed integer value if it is an integer.
    pub fn as_signed_integer(&self) -> Option<i64> {
        match *self {
            State::Signed(i) => Some(i),
            State::Integer(i) => Some(i as i64),
            _ => None,
        }
    }
    /// Extracts the float value if it is a float.
    pub fn as_float(&self) -> Option<f64> {
        match *self {
            State::Float(f) => Some(f),
            State::Signed(i) => Some(i as f64),
            State::Integer(i) => Some(i as f64),
            _ => None,
        }
    }
    /// Extracts the string of this value if it is a string.
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            State::String(ref s) => Some(&**s),
            _ => None,
        }
    }
    /// Extracts the array value if it is an array.
    pub fn as_array(&self) -> Option<&Vec<State>> {
        match *self {
            State::Array(ref s) => Some(s),
            _ => None,
        }
    }
    /// Extracts the hashmap value if it is an hashmap.
    pub fn as_object(&self) -> Option<&HashMap<String, State>> {
        match self {
            State::Object(map) => Some(map),
            _ => None,
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
        let state = State::Boolean(true);
        assert_eq!(state.as_boolean(), Some(true));

        let state = State::Integer(42);
        assert_eq!(state.as_boolean(), None);
    }

    #[test]
    fn test_as_integer() {
        let state = State::Integer(42);
        assert_eq!(state.as_integer(), Some(42));

        let state = State::Boolean(true);
        assert_eq!(state.as_integer(), None);
    }

    #[test]
    fn test_as_signed_integer() {
        let state = State::Signed(-42);
        assert_eq!(state.as_signed_integer(), Some(-42));

        let state = State::Boolean(true);
        assert_eq!(state.as_signed_integer(), None);
    }

    #[test]
    fn test_as_float() {
        let state = State::Float(3.14);
        assert_eq!(state.as_float(), Some(3.14));

        let state = State::Boolean(true);
        assert_eq!(state.as_float(), None);
    }

    #[test]
    fn test_as_str() {
        let state = State::String("test".into());
        assert_eq!(state.as_str(), Some("test"));

        let state = State::Boolean(true);
        assert_eq!(state.as_str(), None);
    }

    #[test]
    fn test_as_array() {
        let state = State::Array(vec![State::Integer(1), State::Integer(2)]);
        assert_eq!(
            state.as_array(),
            Some(&vec![State::Integer(1), State::Integer(2)])
        );

        let state = State::Boolean(true);
        assert_eq!(state.as_array(), None);
    }

    #[test]
    fn test_as_object() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), State::Integer(42));

        let state = State::Object(map.clone());
        assert_eq!(state.as_object(), Some(&map));

        let state = State::Boolean(true);
        assert_eq!(state.as_object(), None);
    }
}
