use std::any::{Any, type_name};
use std::collections::HashMap;

use anyhow::Result;

use crate::storage::entity::private_entity::EntityToAny;
use crate::storage::storage::Storage;

// @todo change to &str ?
pub type EntityType = String;
pub type Id = usize;

#[typetag::serde(tag = "type")]
pub trait Entity: Any + Send + Sync + EntityToAny {
    /// Exposes the entity id.
    fn get_id(&self) -> Id;

    /// (internal)
    /// /!\ You should never use this.
    fn set_id(&mut self, id: Id);

    /// (internal)
    /// Workaround: We would need custom implementation of serialize/deserialize for storage.
    /// In the absence of a found solution at the moment, this method is used to post-process the deserialized of entities.
    /// @todo find a better solution
    /// @todo remove when https://github.com/serde-rs/serde/issues/626
    /// @todo remove when https://github.com/serde-rs/serde/issues/642
    fn post_load(&mut self) {
        // Do nothing by default
    }

    /// Retrieves the entity type.
    fn get_entity_type() -> EntityType
    where
        Self: Sized,
    {
        type_name::<Self>().split("::").last().unwrap().to_string()
    }

    /// List all entities of this kind stored in the storage.
    ///
    /// # Example
    /// ```
    /// use hermes_core::hardware::boards::arduino::{ArduinoBoard, ArduinoModel};
    /// use hermes_core::hardware::boards::Board;
    /// use hermes_core::storage::entity::Entity;
    /// use hermes_core::storage::storage::Storage;
    /// Storage::init_volatile().expect("Storage init");
    /// ArduinoBoard::build("Board1", ArduinoModel::MEGA).save().expect("Error saving the entity");
    /// ArduinoBoard::build("Board2", ArduinoModel::MEGA).save().expect("Error saving the entity");
    /// let boards = Board::list();
    /// assert!(boards.is_ok());
    /// assert_eq!(boards.unwrap().keys().len(), 2);
    /// ```
    fn list() -> Result<HashMap<Id, Self>>
    where
        Self: Sized + Clone,
    {
        Storage::list::<Self>()
    }

    /// Find entity by Id.
    fn get(id: &Id) -> Result<Option<Self>>
    where
        Self: Sized + Clone,
    {
        Storage::get::<Self>(id)
    }

    /// Saves the entity into the storage.
    fn save(self) -> Result<Self>
    where
        Self: Sized + Clone,
    {
        Storage::set(self)
    }

    /// Delete the entity from the storage.
    fn delete(self) -> Result<Option<Self>>
    where
        Self: Sized + Clone,
    {
        Storage::delete::<Self>(self.get_id())
    }

    /// Delete the entity from the storage.
    fn delete_by_id(id: Id) -> Result<Option<Self>>
    where
        Self: Sized + Clone,
    {
        Storage::delete::<Self>(id)
    }
}

pub(crate) mod private_entity {
    use std::any::Any;

    pub trait EntityToAny: 'static {
        fn as_any(&self) -> &dyn Any;
    }

    impl<T: 'static> EntityToAny for T {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use hermes_macros::Entity;

    use crate::storage::entity::Entity;
    use crate::storage::storage::Storage;

    #[Entity]
    #[derive(Clone, Default, Debug, PartialEq)]
    pub struct TestEntity {
        pub dummy: u8,
    }

    // Test getting entity type
    #[test]
    #[serial_test::serial]
    fn test_get_entity_type() {
        assert_eq!(TestEntity::get_entity_type(), "TestEntity");
    }

    // Test saving entity
    #[test]
    #[serial_test::serial]
    fn test_save_entity() {
        let _ = Storage::init_volatile();
        let entity = TestEntity::default();
        assert!(entity.save().is_ok());
    }

    // Test getting entity by id
    #[test]
    #[serial_test::serial]
    fn test_get_entity_by_id() {
        let _ = Storage::init_volatile();
        let entity = TestEntity::default();
        let saved_entity = entity.save().unwrap();
        let retrieved_entity = TestEntity::get(&saved_entity.get_id()).unwrap().unwrap();
        assert_eq!(saved_entity, retrieved_entity);
    }

    // Test list entity
    #[test]
    #[serial_test::serial]
    fn test_list_entity() {
        let _ = Storage::init_volatile();
        TestEntity::default().save().expect("panic");
        TestEntity::default().save().expect("panic");
        assert!(TestEntity::list().is_ok());
        assert_eq!(TestEntity::list().unwrap().keys().len(), 2);
    }

    // Test delete entity
    #[test]
    #[serial_test::serial]
    fn test_delete_entity() {
        let _ = Storage::init_volatile();
        TestEntity::default().save().expect("panic");
        let to_be_deleted = TestEntity::default().save().expect("panic");
        let to_be_deleted_by_id = TestEntity::default().save().expect("panic");
        assert_eq!(TestEntity::list().unwrap().keys().len(), 3);

        assert!(to_be_deleted.delete().is_ok());
        assert_eq!(TestEntity::list().unwrap().keys().len(), 2);

        assert!(TestEntity::delete_by_id(to_be_deleted_by_id.get_id()).is_ok());
        assert_eq!(TestEntity::list().unwrap().keys().len(), 1);
    }
}
