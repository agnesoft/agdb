use crate::collections::multi_map::MultiMapStorage;
use crate::collections::vec::DbVec;
use crate::collections::vec::VecValue;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::DbError;
use crate::DbId;
use crate::DbValue;
use crate::StorageData;

#[derive(Copy, Clone)]
pub struct DbIndexStorageIndex {
    key_index: DbValueIndex,
    ids_index: StorageIndex,
}

#[allow(dead_code)]
pub struct DbIndex<D>
where
    D: StorageData,
{
    key: DbValue,
    key_index: DbValueIndex,
    ids: MultiMapStorage<DbValue, DbId, D>,
}

#[allow(dead_code)]
pub struct DbIndexes<D>
where
    D: StorageData,
{
    indexes: Vec<DbIndex<D>>,
    storage_indexes: DbVec<DbIndexStorageIndex, D>,
}

#[allow(dead_code)]
impl<D> DbIndex<D>
where
    D: StorageData,
{
    pub fn from_storage(storage: &Storage<D>, index: DbIndexStorageIndex) -> Result<Self, DbError> {
        let key_index = index.key_index;
        let key = DbValue::load_db_value(key_index, storage)?;
        let ids = MultiMapStorage::from_storage(storage, index.ids_index)?;

        Ok(Self {
            key_index,
            key,
            ids,
        })
    }

    pub fn ids(&self) -> &MultiMapStorage<DbValue, DbId, D> {
        &self.ids
    }

    pub fn ids_mut(&mut self) -> &mut MultiMapStorage<DbValue, DbId, D> {
        &mut self.ids
    }

    pub fn key(&self) -> &DbValue {
        &self.key
    }

    pub fn new(key: DbValue, storage: &mut Storage<D>) -> Result<Self, DbError> {
        let key_index = key.store_db_value(storage)?;
        let ids = MultiMapStorage::new(storage)?;

        Ok(Self {
            key,
            key_index,
            ids,
        })
    }

    pub fn remove_from_storage(self, storage: &mut Storage<D>) -> Result<(), DbError> {
        let id = storage.transaction();
        DbValue::remove(storage, &self.key_index.data())?;
        self.ids.remove_from_storage(storage)?;
        storage.commit(id)
    }

    pub fn storage_index(&self) -> DbIndexStorageIndex {
        DbIndexStorageIndex {
            key_index: self.key_index,
            ids_index: self.ids.storage_index(),
        }
    }
}

#[allow(dead_code)]
impl<D> DbIndexes<D>
where
    D: StorageData,
{
    pub fn from_storage(storage: &Storage<D>, index: StorageIndex) -> Result<Self, DbError> {
        let storage_indexes: DbVec<DbIndexStorageIndex, D> = DbVec::from_storage(storage, index)?;
        let mut indexes = vec![];

        for storage_index in storage_indexes.iter(storage) {
            let index = DbIndex::from_storage(storage, storage_index)?;
            indexes.push(index);
        }

        Ok(Self {
            indexes,
            storage_indexes,
        })
    }

    pub fn index(&self, key: &DbValue) -> Option<&DbIndex<D>> {
        self.indexes.iter().find(|index| index.key() == key)
    }

    pub fn index_mut(&mut self, key: &DbValue) -> Option<&mut DbIndex<D>> {
        self.indexes.iter_mut().find(|index| index.key() == key)
    }

    pub fn indexes(&self) -> &[DbIndex<D>] {
        self.indexes.as_slice()
    }

    pub fn insert(
        &mut self,
        storage: &mut Storage<D>,
        key: DbValue,
    ) -> Result<&mut DbIndex<D>, DbError> {
        let index = DbIndex::new(key, storage)?;
        self.storage_indexes.push(storage, &index.storage_index())?;
        self.indexes.push(index);
        return Ok(self.indexes.last_mut().unwrap());
    }

    pub fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        Ok(Self {
            indexes: vec![],
            storage_indexes: DbVec::new(storage)?,
        })
    }

    pub fn remove(&mut self, storage: &mut Storage<D>, key: &DbValue) -> Result<(), DbError> {
        if let Some(pos) = self.indexes.iter().position(|index| index.key() == key) {
            self.storage_indexes.remove(storage, pos as u64)?;
            let index = self.indexes.remove(pos);
            index.remove_from_storage(storage)?;
        }

        Ok(())
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_indexes.storage_index()
    }
}

impl VecValue for DbValue {
    fn store<D: StorageData>(
        &self,
        storage: &mut crate::storage::Storage<D>,
    ) -> Result<Vec<u8>, crate::DbError> {
        let index = self.store_db_value(storage)?;
        Ok(index.data().to_vec())
    }

    fn load<D: StorageData>(
        storage: &crate::storage::Storage<D>,
        bytes: &[u8],
    ) -> Result<Self, crate::DbError> {
        let index = DbValueIndex::deserialize(bytes)?;
        DbValue::load_db_value(index, storage)
    }

    fn remove<D: StorageData>(
        storage: &mut crate::storage::Storage<D>,
        bytes: &[u8],
    ) -> Result<(), crate::DbError> {
        let index = DbValueIndex::deserialize(bytes)?;

        if !index.is_value() {
            storage.remove(StorageIndex(index.index()))?;
        }

        Ok(())
    }

    fn storage_len() -> u64 {
        DbValueIndex::serialized_size_static()
    }
}

impl VecValue for DbIndexStorageIndex {
    fn store<D: StorageData>(&self, _storage: &mut Storage<D>) -> Result<Vec<u8>, DbError> {
        let key_index = self.key_index.serialize();
        let ids_index = self.ids_index.serialize();
        Ok([key_index, ids_index].concat())
    }

    fn load<D: StorageData>(_storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError> {
        let key_index = DbValueIndex::deserialize(bytes)?;
        let ids_index = StorageIndex::deserialize(&bytes[key_index.serialized_size() as usize..])?;
        Ok(Self {
            key_index,
            ids_index,
        })
    }

    fn remove<D: StorageData>(_storage: &mut Storage<D>, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        DbValueIndex::serialized_size_static() + StorageIndex::serialized_size_static()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryStorage;

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn derived_from_clone() {
        let left = DbIndexStorageIndex {
            key_index: DbValueIndex::new(),
            ids_index: StorageIndex::from(1),
        };
        let other = left.clone();
        assert_eq!(left.key_index, other.key_index);
        assert_eq!(left.ids_index, other.ids_index);
    }

    #[test]
    fn from_storage() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let index;

        {
            let mut indexes = DbIndexes::new(&mut storage).unwrap();
            indexes.insert(&mut storage, "username".into()).unwrap();
            indexes.insert(&mut storage, "token".into()).unwrap();
            let username_index = indexes.index_mut(&"username".into()).unwrap();
            username_index
                .ids_mut()
                .insert(&mut storage, &"user1".into(), &DbId(5))
                .unwrap();
            index = indexes.storage_index();
        }

        let indexes = DbIndexes::from_storage(&storage, index).unwrap();

        assert_eq!(
            indexes
                .index(&"username".into())
                .unwrap()
                .ids()
                .value(&storage, &"user1".into())
                .unwrap(),
            Some(DbId(5))
        );
    }

    #[test]
    fn index() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut indexes = DbIndexes::new(&mut storage).unwrap();
        indexes.insert(&mut storage, "username".into()).unwrap();
        indexes.insert(&mut storage, "token".into()).unwrap();
        assert_eq!(
            indexes.index(&"username".into()).unwrap().key,
            "username".into()
        );
        assert_eq!(indexes.index(&"token".into()).unwrap().key, "token".into());
    }

    #[test]
    fn index_missing() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut indexes = DbIndexes::new(&mut storage).unwrap();
        indexes.insert(&mut storage, "username".into()).unwrap();
        assert!(indexes.index(&"token".into()).is_none());
    }

    #[test]
    fn index_mut() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut indexes = DbIndexes::new(&mut storage).unwrap();
        indexes.insert(&mut storage, "username".into()).unwrap();
        indexes.insert(&mut storage, "token".into()).unwrap();
        assert_eq!(
            indexes.index_mut(&"username".into()).unwrap().key,
            "username".into()
        );
        assert_eq!(
            indexes.index_mut(&"token".into()).unwrap().key,
            "token".into()
        );
    }

    #[test]
    fn index_mut_missing() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut indexes = DbIndexes::new(&mut storage).unwrap();
        indexes.insert(&mut storage, "username".into()).unwrap();
        assert!(indexes.index_mut(&"token".into()).is_none());
    }

    #[test]
    fn insert() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut indexes = DbIndexes::new(&mut storage).unwrap();
        let index = indexes.insert(&mut storage, "username".into()).unwrap();
        index
            .ids_mut()
            .insert(&mut storage, &"user1".into(), &DbId(1))
            .unwrap();
        let id = index
            .ids()
            .value(&storage, &"user1".into())
            .unwrap()
            .unwrap();
        assert_eq!(id, DbId(1));
    }

    #[test]
    fn remove() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut indexes = DbIndexes::new(&mut storage).unwrap();
        indexes
            .insert(&mut storage, "username_which_is_long".into())
            .unwrap()
            .ids_mut()
            .insert(&mut storage, &"".into(), &DbId(0))
            .unwrap();
        indexes
            .remove(&mut storage, &"username_which_is_long".into())
            .unwrap();
        assert!(indexes.index(&"username_which_is_long".into()).is_none());
    }

    #[test]
    fn remove_from_storage_db_index() {
        let mut storage: Storage<MemoryStorage> = Storage::new("test").unwrap();
        let mut index =
            DbIndex::new("username which is not exactly short".into(), &mut storage).unwrap();
        index
            .ids_mut()
            .insert(
                &mut storage,
                &"user some really long username".into(),
                &DbId(1),
            )
            .unwrap();

        index.remove_from_storage(&mut storage).unwrap();

        assert_ne!(storage.len(), 0);

        storage.shrink_to_fit().unwrap();

        assert_eq!(storage.len(), 0)
    }

    #[test]
    fn remove_missing() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut indexes = DbIndexes::new(&mut storage).unwrap();
        indexes.remove(&mut storage, &"username".into()).unwrap();
        assert!(indexes.index(&"username".into()).is_none());
    }
}
