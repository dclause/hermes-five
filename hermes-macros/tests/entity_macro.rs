#![cfg(feature = "serde")]

extern crate hermes_macros;

use hermes_five::storage::{entity::Entity as EntityTrait, entity::Id};
use hermes_macros::Entity;

#[Entity]
struct MyEntity {
    name: String,
    value: i32,
}

#[test]
fn test_entity_macro() {
    let mut entity = MyEntity {
        name: "Test".to_string(),
        value: 42,
        id: Id::from(42u8),
    };

    // Test get_id
    let id = entity.get_id();
    assert_eq!(id, entity.id);

    // Test set_id
    let new_id = Id::from(69u8);
    entity.set_id(new_id);
    assert_eq!(entity.get_id(), new_id);

    // Test serialization
    let serialized = serde_json::to_string(&entity).unwrap();
    let deserialized: MyEntity = serde_json::from_str(&serialized).unwrap();
    assert_eq!(entity.id, deserialized.id);
    assert_eq!(entity.name, deserialized.name);
    assert_eq!(entity.value, deserialized.value);
}

#[cfg(test)]
mod tests {
    use trybuild::TestCases;

    #[test]
    fn test_compile_failures() {
        let t = TestCases::new();
        t.compile_fail("tests/compile_fail/incorrect_entity.rs");
    }

    #[test]
    fn test_entity_macro() {
        super::test_entity_macro();
    }
}
