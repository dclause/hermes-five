//! This module contains all the code to define `Storage`.
//!
//! The principle of storage is to provide a structure where elements implementing the `Entity` trait
//! can be stored.
//!
//! The storage is globally accessible by other services. For instance the socket and rest API
//! can't work if no storage of some sort is provided.
//!
//! Currently, the Storage comes in two flavor:
//! - volatile: purely in memory (thus resets at every app start)
//! - persistent: the data are serialized in JSON file

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, bail, Result};

use crate::storage::entity::{Entity, EntityType, Id};
use crate::utils::file::resolve_file;

static STORAGE: RwLock<Option<Storage>> = RwLock::new(None);

/// Storage structure: stores all data accessible via the API.
#[derive(Default)]
pub struct Storage {
    /// Path to the destination storage folder.
    /// If given, the `.save()` function will be able to serialize the content into that folder.
    destination: Option<PathBuf>,
    /// Flag to indicate if `.save()` should be automatically done after every CRUD operation.
    /// By opposition, when set to `false` no persistent storage is done without a manual call to `.save()`.
    autosave: bool,
    /// Stores the entities in memory.
    entities: Arc<RwLock<HashMap<EntityType, HashMap<Id, Box<dyn Entity>>>>>,
}

impl Storage {
    /// Checks if the storage has already been initialized.
    pub fn is_init() -> bool {
        STORAGE.read().unwrap().is_some()
    }

    /// Initializes the storage a volatile 'in-memory' only storage.
    pub fn init_volatile() -> Result<()> {
        let mut w = STORAGE.write().unwrap();
        *w = Some(Storage::default());
        Ok(())
    }

    /// Initializes the storage a persisted 'in-file' storage.
    ///
    /// # Parameters
    /// * `folder`: A ref to a `Path`  
    ///     * If the `folder` does not exist: it gets created and initializes empty.  
    ///     * If the `folder` exists: act according to `reset_if_exists` parameter.
    /// * `reset_if_exists`: A boolean flag.
    ///     * `true`: the folder is emptied.  
    ///     * `false`: the content is extracted from the files into the Storage.  
    /// * `autosave`: A boolean flag.
    ///     * `true`: the storage content will be dumped
    ///       to the file system everytime something relevant is created / updated / deleted.  
    ///     * `false`: the storage will work exclusively
    ///       in memory (faster) but it is the programmer responsibility to save it manually
    ///       via the `save` method.
    ///
    /// # Returns
    /// `Result<()>`: the function may throw various errors with explicit text message.
    pub fn init_persistent<P: AsRef<Path>>(
        folder: P,
        reset_if_exists: bool,
        autosave: bool,
    ) -> Result<()> {
        let path = resolve_file(folder)?.join("storage");

        // Check path validity.
        if !path.is_dir() {
            if path.exists() {
                bail!("Provided destination is not a directory.");
            }
            std::fs::create_dir_all(&path)?;
        }

        let mut storage = Storage {
            destination: Some(path.clone()),
            autosave,
            entities: Default::default(),
        };

        // Reset the storage if necessary / Load content otherwise.
        if reset_if_exists {
            std::fs::remove_dir_all(&path)?;
            std::fs::create_dir_all(&path)?;
        } else {
            storage = storage.load()?;
        }

        // Set globally.
        let mut w = STORAGE.write().unwrap();
        *w = Some(storage);

        Ok(())
    }

    /// Dumps the storage content to the persistent storage if available.
    ///
    /// If no destination was given when the storage was created
    /// (using `Storage::from(...)`) an `Err` is raised.
    pub fn dump() -> Result<()> {
        let lock = STORAGE
            .read()
            .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;
        let storage = lock
            .as_ref()
            .ok_or_else(|| anyhow!("Storage not initialized"))?;

        if let Some(destination) = &storage.destination {
            let entities = storage
                .entities
                .read()
                .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;

            for (entity_type, entities_of_type) in entities.iter() {
                let serialized_entities = serde_json::to_string_pretty(entities_of_type)?;

                let entity_filepath = destination.join(format!("{}.json", entity_type));
                let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(entity_filepath)?;
                file.write_all(serialized_entities.as_ref())?;
            }
        }

        Ok(())
    }

    /// (private)
    /// Loads content from the persistent storage if available.
    fn load(self) -> Result<Self> {
        if let Some(destination) = &self.destination {
            let files = std::fs::read_dir(destination)?;

            for file in files {
                let file = file?.path();
                let entity_type: EntityType =
                    file.file_stem().unwrap().to_str().unwrap().to_string();

                // Deserialize data from the current entity_type file in the storage folder.
                let data = std::fs::read_to_string(file)?;
                let mut entities =
                    serde_json::from_str::<HashMap<Id, Box<dyn Entity>>>(data.as_str())?;

                // Call the post_load hook on each loaded entities.
                for (_, entity) in entities.iter_mut() {
                    entity.post_load();
                }

                // Update the storage to save the provided entity
                self.entities.write().unwrap().insert(entity_type, entities);
            }
        }
        Ok(self)
    }

    /// Retrieves all the entities of an entity_type stored in the storage.
    ///
    /// This method is private to this crate only since developers should rather
    /// use the entities CRUD methods instead of directly using the storage.
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// use hermes_core::hardware::boards::arduino::{ArduinoBoard, ArduinoModel};
    /// use hermes_core::hardware::boards::Board;
    /// use hermes_core::storage::entity::{Entity, Id};
    /// use hermes_core::storage::storage::Storage;
    /// Storage::init_volatile().expect("Storage init");
    /// ArduinoBoard::build("Board1", ArduinoModel::MEGA).save().expect("Error saving the entity");
    /// ArduinoBoard::build("Board2", ArduinoModel::MEGA).save().expect("Error saving the entity");
    /// let boards: HashMap<Id, Board> = Board::list().expect("panic!");
    /// assert_eq!(boards.keys().len(), 2);
    /// ```
    pub(crate) fn list<T: Entity + Clone + 'static>() -> Result<HashMap<Id, T>> {
        let lock = STORAGE
            .read()
            .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;
        let storage = lock
            .as_ref()
            .ok_or_else(|| anyhow!("Storage not initialized"))?;

        let lock = storage
            .entities
            .read()
            .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;

        let entity_type = T::get_entity_type();
        let entities = lock.get(&entity_type).map_or(HashMap::new(), |entities| {
            entities
                .iter()
                .map(|(id, entity)| {
                    let entity = entity.deref().as_any().downcast_ref::<T>();
                    (*id, entity.unwrap().clone())
                })
                .collect()
        });

        Ok(entities)
    }

    /// Retrieves an entity stored in the storage.
    ///
    /// This method is private to this crate only since developers should rather
    /// use the entities CRUD methods instead of directly using the storage.
    ///
    /// # Example
    /// ```
    /// use hermes_core::hardware::boards::arduino::{ArduinoBoard, ArduinoModel};
    /// use hermes_core::hardware::boards::Board;
    /// use hermes_core::storage::entity::Entity;
    /// use hermes_core::storage::storage::Storage;
    /// Storage::init_volatile().expect("Storage init");
    /// let entity = ArduinoBoard::build("Board", ArduinoModel::MEGA).save().expect("Error saving the entity");
    /// let retrieved_entity = Board::get(&entity.get_id()).expect("Error using the storage");
    /// assert!(retrieved_entity.is_some());
    /// ```
    pub fn get<T: Entity + Clone + 'static>(id: &Id) -> Result<Option<T>> {
        let lock = STORAGE
            .read()
            .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;
        let storage = lock
            .as_ref()
            .ok_or_else(|| anyhow!("Storage not initialized"))?;

        let lock = storage
            .entities
            .read()
            .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;

        let entity_type = T::get_entity_type();

        let entity = lock
            .get(&entity_type)
            .and_then(|entities| entities.get(id))
            .map_or(None, |entity| {
                let entity = entity.deref().as_any().downcast_ref::<T>();
                entity.cloned()
            });

        Ok(entity)
    }

    /// Stores or Updates an entity stored in the storage.
    ///
    /// This method is private to this crate only since developers should rather
    /// use the entities CRUD methods instead of directly using the storage.
    ///
    /// # Example
    /// ```
    /// use hermes_core::hardware::boards::arduino::{ArduinoBoard, ArduinoModel};
    /// use hermes_core::hardware::boards::Board;
    /// use hermes_core::storage::entity::Entity;
    /// use hermes_core::storage::storage::Storage;
    /// Storage::init_volatile().expect("Storage init");
    /// let entity = ArduinoBoard::build("Board", ArduinoModel::MEGA);
    /// assert_eq!(entity.get_id(), 0);  // Ids are 0 by default, ie not saved.
    /// let saved_entity = entity.save().expect("Storage failed");
    /// assert!(saved_entity.get_id() != 0); // Saving for the first time will give the entity a none null id.
    /// ```
    pub(crate) fn set<T: Entity + 'static + Clone>(mut entity: T) -> Result<T> {
        let lock = STORAGE
            .read()
            .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;
        let storage = lock
            .as_ref()
            .ok_or_else(|| anyhow!("Storage not initialized"))?;

        let mut lock = storage
            .entities
            .write()
            .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;

        let entity_type = T::get_entity_type();
        let entities = lock.entry(entity_type.clone()).or_insert_with(HashMap::new);

        // Get the entity id or generate if not set.
        let id = match entity.get_id() {
            0 => entities.keys().max().map_or(1, |id| id + 1),
            id => id,
        };

        entity.set_id(id);
        entities.insert(id, Box::new(entity.clone()));

        // Optionally, triggers autosave if enabled:
        // this will save the entities of the current type to its dump file.
        if storage.autosave {
            let path = storage
                .destination
                .as_ref()
                .ok_or_else(|| anyhow!("Destination folder undefined."))?;

            let serialized_entities = serde_json::to_string_pretty(entities)?;

            let entity_filepath = path.join(format!("{}.json", entity_type));
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(entity_filepath)?;
            file.write_all(serialized_entities.as_ref())?;
        }

        Ok(entity)
    }

    /// Deletes an entity stored from the storage.
    ///
    /// This method is private to this crate only since developers should rather
    /// use the entities CRUD methods instead of directly using the storage.
    ///
    /// # Example
    /// ```
    /// use hermes_core::hardware::boards::arduino::{ArduinoBoard, ArduinoModel};
    /// use hermes_core::hardware::boards::Board;
    /// use hermes_core::storage::entity::Entity;
    /// use hermes_core::storage::storage::Storage;
    /// Storage::init_volatile().expect("Storage init");
    ///
    /// // Delete from the entity.
    /// let entity = ArduinoBoard::build("Board", ArduinoModel::MEGA).save().expect("panic!");
    /// assert!(entity.delete().is_ok());
    ///
    /// // Static delete by Id is also possible.
    /// let entity = ArduinoBoard::build("Board", ArduinoModel::MEGA).save().expect("panic!");
    /// assert!(Board::delete_by_id(entity.get_id()).is_ok());
    /// ```
    pub(crate) fn delete<T: Entity + 'static + Clone>(id: Id) -> Result<Option<T>> {
        let lock = STORAGE
            .read()
            .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;
        let storage = lock
            .as_ref()
            .ok_or_else(|| anyhow!("Storage not initialized"))?;

        let mut lock = storage
            .entities
            .write()
            .map_err(|err| anyhow!("Storage lock cannot be acquired: {}", err))?;

        let entity_type = T::get_entity_type();
        let entities = lock.entry(entity_type.clone()).or_insert_with(HashMap::new);

        let entity = entities.remove(&id).map_or(None, |entity| {
            let entity = entity.deref().as_any().downcast_ref::<T>();
            entity.cloned()
        });

        // Optionally, triggers autosave if enabled:
        // this will save the entities of the current type to its dump file.
        if entity.is_some() && storage.autosave {
            let path = storage
                .destination
                .as_ref()
                .ok_or_else(|| anyhow!("Destination folder undefined."))?;

            let serialized_entities = serde_json::to_string_pretty(entities)?;

            let entity_filepath = path.join(format!("{}.json", entity_type));
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(entity_filepath)?;
            file.write_all(serialized_entities.as_ref())?;
        }

        Ok(entity)
    }

    // /// Helper to debug the storage content.
    // @todo: that would impose Entity to impl Debug:
    //        reconsider when https://github.com/rust-lang/rust/issues/31844
    // pub fn debug() {
    //     if let Some(binding) = STORAGE.get() {
    //         if let Ok(entities) = binding.entities.read() {
    //             println!("Storage contents:");
    //             for (entity_type, entity_map) in entities.iter() {
    //                 println!("  EntityType: {:?}", entity_type);
    //                 for (id, entity) in entity_map {
    //                     println!("    ID: {:?}", id);
    //                     // Debug entity if Entity implements Debug
    //                     println!("      {:?}", entity);
    //                 }
    //             }
    //         }
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use std::fs::remove_dir_all;

    use hermes_macros::Entity;

    use super::*;

    #[Entity]
    #[derive(Clone, Default, Debug, PartialEq)]
    struct TestStorage {}

    // Test volatile storage initialization
    #[test]
    #[serial_test::serial(storage)]
    fn test_init_volatile() {
        assert!(Storage::init_volatile().is_ok());
    }

    // Test persistent storage initialization with reset
    #[test]
    #[serial_test::serial(storage)]
    fn test_init_persistent_with_reset() {
        assert!(Storage::init_persistent(
            "./tests/generated/test_init_persistent_with_reset",
            true,
            true
        )
        .is_ok());
    }

    // Test persistent storage initialization without reset
    #[test]
    #[serial_test::serial(storage)]
    fn test_init_persistent_without_reset() {
        let storage = Storage::init_persistent(
            "./tests/generated/test_init_persistent_without_reset",
            false,
            true,
        );
        assert!(storage.is_ok(), "{:?}", storage);
    }

    // Test getting an entity
    #[test]
    #[serial_test::serial(storage)]
    fn test_get_entity() {
        Storage::init_volatile().expect("panic");
        let entity = TestStorage::default();
        let saved_entity = Storage::set(entity.clone());
        assert!(saved_entity.is_ok());
        let saved_entity = saved_entity.unwrap();
        let retrieved_entity = Storage::get::<TestStorage>(&saved_entity.get_id());
        assert!(retrieved_entity.is_ok());
        let retrieved_entity = retrieved_entity.unwrap();
        assert!(retrieved_entity.is_some());
        assert_eq!(saved_entity, retrieved_entity.unwrap());
    }

    // Test setting an entity
    #[test]
    #[serial_test::serial(storage)]
    fn test_set_entity() {
        Storage::init_volatile().expect("panic");
        let entity = TestStorage::default();
        assert_eq!(entity.get_id(), 0);
        let saved_entity = Storage::set(entity.clone()).unwrap();
        assert_ne!(saved_entity.get_id(), 0);
    }

    // Integration test: Test setting and getting an entity
    #[test]
    #[serial_test::serial(storage)]
    fn test_set_and_get_entity() {
        Storage::init_volatile().expect("panic");
        let entity = TestStorage::default();
        let saved_entity = Storage::set(entity.clone()).unwrap();
        let retrieved_entity = Storage::get::<TestStorage>(&saved_entity.get_id())
            .unwrap()
            .unwrap();
        assert_eq!(saved_entity, retrieved_entity);
    }

    // Integration test: Test setting and getting multiple entities
    #[test]
    #[serial_test::serial(storage)]
    fn test_set_and_get_multiple_entities() {
        Storage::init_volatile().expect("panic");
        let entity1 = TestStorage::default();
        let entity2 = TestStorage::default();
        let saved_entity1 = Storage::set(entity1.clone()).unwrap();
        let saved_entity2 = Storage::set(entity2.clone()).unwrap();
        let retrieved_entity1 = Storage::get::<TestStorage>(&saved_entity1.get_id())
            .unwrap()
            .unwrap();
        let retrieved_entity2 = Storage::get::<TestStorage>(&saved_entity2.get_id())
            .unwrap()
            .unwrap();
        assert_eq!(saved_entity1, retrieved_entity1);
        assert_eq!(saved_entity2, retrieved_entity2);
        assert_ne!(saved_entity1.get_id(), saved_entity2.get_id());
    }

    // Test dumping storage content
    #[test]
    #[serial_test::serial(storage)]
    fn test_persistence() {
        Storage::init_volatile().expect("panic");
        TestStorage::default().save().expect("panic");
        assert!(Storage::dump().is_ok());

        let dump_folder = resolve_file("./tests/generated/test_persistence").unwrap();
        let dump_file =
            resolve_file("./tests/generated/test_persistence/storage/TestStorage.json").unwrap();
        if dump_folder.is_dir() {
            remove_dir_all(dump_folder.clone()).expect("panic")
        };

        // Test folder creation when initialize storage.
        assert_eq!(dump_folder.is_dir(), false);
        let _ = Storage::init_persistent(dump_folder.clone(), true, false);
        assert_eq!(dump_folder.is_dir(), true);

        // Test non-autosave saving.
        TestStorage::default().save().expect("panic");
        assert_eq!(dump_file.exists(), false);

        // Test autosave saving.
        let _ = Storage::init_persistent(dump_folder.clone(), true, true);
        TestStorage::default().save().expect("panic");
        assert_eq!(dump_file.exists(), true);

        // Test dump.
        remove_dir_all(dump_folder.clone()).expect("panic");
        assert_eq!(dump_folder.is_dir(), false);
        assert_eq!(dump_file.exists(), false);
        let _ = Storage::init_persistent(dump_folder.clone(), true, false);
        TestStorage::default().save().expect("panic");
        assert!(Storage::dump().is_ok());
        assert_eq!(dump_folder.is_dir(), true);
        assert_eq!(dump_file.exists(), true);
    }

    // Test debug method
    // #[test]
    // #[serial_test::serial(storage)]
    // fn test_debug() {
    //     let _ = Storage::init_volatile();
    //     let entity = Dummy::default();
    //     let _ = Storage::set(entity.clone()).unwrap();
    //     Storage::debug();
    // }
}
