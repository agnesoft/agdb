use crate::DbError;
use crate::DbValue;
use crate::StorageData;
use crate::collections::vec::DbVec;
use crate::collections::vec::VecValue;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;

/// Database key-value pair (aka property) attached to
/// database elements. It can be constructed from a
/// tuple of types that are convertible to `DbValue`.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct DbKeyValue {
    /// Key of the property
    pub key: DbValue,

    /// Value of the property
    pub value: DbValue,
}

pub struct DbKeyValues<S: StorageData>(DbVec<StorageIndex, S>);

impl<S: StorageData> DbKeyValues<S> {
    pub fn new(storage: &mut Storage<S>) -> Result<Self, DbError> {
        Ok(Self(DbVec::new(storage)?))
    }

    pub fn from_storage(storage: &Storage<S>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(Self(DbVec::from_storage(storage, index)?))
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.0.storage_index()
    }

    pub fn len(&self) -> u64 {
        self.0.len()
    }

    pub fn insert_value(
        &mut self,
        storage: &mut Storage<S>,
        index: u64,
        value: &DbKeyValue,
    ) -> Result<(), DbError> {
        if self.0.len() <= index {
            self.0
                .resize(storage, index + 1, &StorageIndex::default())?;
        }

        let storage_index = self.0.value(storage, index)?;

        let mut kvs = if storage_index.0 == 0 {
            let kvs = DbVec::<DbKeyValue, S>::new(storage)?;
            self.0.replace(storage, index, &kvs.storage_index())?;
            kvs
        } else {
            DbVec::from_storage(storage, storage_index)?
        };

        kvs.push(storage, value)
    }

    pub fn insert_or_replace(
        &mut self,
        storage: &mut Storage<S>,
        index: u64,
        value: &DbKeyValue,
    ) -> Result<Option<DbKeyValue>, DbError> {
        if !self.valid_index(storage, index)? {
            self.insert_value(storage, index, value)?;
            return Ok(None);
        }

        let mut kvs = self.kvs(storage, index)?;

        if let Some((index, kv)) = kvs
            .iter(storage)
            .enumerate()
            .find(|(_, kv)| kv.key == value.key)
        {
            kvs.replace(storage, index as u64, value)?;
            Ok(Some(kv))
        } else {
            kvs.push(storage, value)?;
            Ok(None)
        }
    }

    pub fn key_count(&self, storage: &Storage<S>, index: u64) -> Result<u64, DbError> {
        if !self.valid_index(storage, index)? {
            return Ok(0);
        }

        Ok(self.kvs(storage, index)?.len())
    }

    pub fn keys(&self, storage: &Storage<S>, index: u64) -> Result<Vec<DbValue>, DbError> {
        if !self.valid_index(storage, index)? {
            return Ok(vec![]);
        }

        let kvs = self.kvs(storage, index)?;
        Ok(kvs.iter(storage).map(|kv| kv.key).collect())
    }

    pub fn remove(&mut self, storage: &mut Storage<S>, index: u64) -> Result<(), DbError> {
        if !self.valid_index(storage, index)? {
            return Ok(());
        }

        let kvs = self.kvs(storage, index)?;
        kvs.remove_from_storage(storage)?;

        if self.0.len() - 1 == index {
            self.0.remove(storage, index)?;
        } else {
            self.0.replace(storage, index, &StorageIndex::default())?;
        }

        Ok(())
    }

    pub fn remove_value(
        &mut self,
        storage: &mut Storage<S>,
        index: u64,
        key: &DbValue,
    ) -> Result<(), DbError> {
        if !self.valid_index(storage, index)? {
            return Ok(());
        }

        let mut kvs = self.kvs(storage, index)?;

        if let Some((index, _)) = kvs.iter(storage).enumerate().find(|(_, kv)| kv.key == *key) {
            kvs.remove(storage, index as u64)?;
        }

        Ok(())
    }

    pub fn value(
        &self,
        storage: &Storage<S>,
        index: u64,
        key: &DbValue,
    ) -> Result<Option<DbValue>, DbError> {
        if !self.valid_index(storage, index)? {
            return Ok(None);
        }

        let kvs = self.kvs(storage, index)?;
        Ok(kvs
            .iter(storage)
            .find(|kv| kv.key == *key)
            .map(|kv| kv.value))
    }

    pub fn values(&self, storage: &Storage<S>, index: u64) -> Result<Vec<DbKeyValue>, DbError> {
        if !self.valid_index(storage, index)? {
            return Ok(vec![]);
        }

        let kvs = self.kvs(storage, index)?;
        Ok(kvs.iter(storage).collect())
    }

    pub fn values_by_keys(
        &self,
        storage: &Storage<S>,
        index: u64,
        keys: &[DbValue],
    ) -> Result<Vec<DbKeyValue>, DbError> {
        if !self.valid_index(storage, index)? {
            return Ok(vec![]);
        }

        let kvs = self.kvs(storage, index)?;
        let mut values = kvs
            .iter(storage)
            .filter_map(|kv| keys.iter().position(|k| k == &kv.key).map(|pos| (pos, kv)))
            .collect::<Vec<_>>();
        values.sort_by_key(|(i, _)| *i);
        Ok(values.into_iter().map(|(_, kv)| kv).collect())
    }

    fn kvs(&self, storage: &Storage<S>, index: u64) -> Result<DbVec<DbKeyValue, S>, DbError> {
        let storage_index = self.0.value(storage, index)?;
        DbVec::from_storage(storage, storage_index)
    }

    fn valid_index(&self, storage: &Storage<S>, index: u64) -> Result<bool, DbError> {
        Ok(index < self.0.len() && self.0.value(storage, index)?.0 != 0)
    }
}

impl<K, T> From<(K, T)> for DbKeyValue
where
    K: Into<DbValue>,
    T: Into<DbValue>,
{
    fn from(value: (K, T)) -> Self {
        DbKeyValue {
            key: value.0.into(),
            value: value.1.into(),
        }
    }
}

impl<D: StorageData> VecValue<D> for DbKeyValue {
    fn store(&self, storage: &mut Storage<D>) -> Result<Vec<u8>, DbError> {
        let key_index = self.key.store_db_value(storage)?;
        let value_index = self.value.store_db_value(storage)?;
        Ok([key_index.data(), value_index.data()].concat())
    }

    fn load(storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError> {
        let key_index = DbValueIndex::deserialize(bytes)?;
        let value_index =
            DbValueIndex::deserialize(&bytes[key_index.serialized_size() as usize..])?;
        let key = DbValue::load_db_value(key_index, storage)?;
        let value = DbValue::load_db_value(value_index, storage)?;
        Ok(Self { key, value })
    }

    fn remove(storage: &mut Storage<D>, bytes: &[u8]) -> Result<(), DbError> {
        let key_index = DbValueIndex::deserialize(bytes)?;
        let value_index =
            DbValueIndex::deserialize(&bytes[key_index.serialized_size() as usize..])?;

        if !key_index.is_value() {
            storage.remove(StorageIndex(key_index.index()))?;
        }

        if !value_index.is_value() {
            storage.remove(StorageIndex(value_index.index()))?;
        }

        Ok(())
    }

    fn storage_len() -> u64 {
        DbValueIndex::serialized_size_static() * 2
    }
}

#[cfg(test)]
mod tests {
    use crate::MemoryStorage;

    use super::*;
    #[test]
    fn derived_from_debug() {
        let _ = format!(
            "{:?}",
            DbKeyValue {
                key: DbValue::I64(0),
                value: DbValue::I64(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbKeyValue {
                key: DbValue::I64(0),
                value: DbValue::I64(0)
            },
            DbKeyValue {
                key: DbValue::I64(0),
                value: DbValue::I64(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_ord() {
        let element = DbKeyValue {
            key: DbValue::I64(0),
            value: DbValue::I64(0),
        };
        let other = DbKeyValue {
            key: DbValue::I64(0),
            value: DbValue::I64(0),
        };
        assert!(element <= other);
    }

    #[test]
    fn derived_from_ord() {
        let element = DbKeyValue {
            key: DbValue::I64(0),
            value: DbValue::I64(0),
        };
        assert_eq!(element.cmp(&element), std::cmp::Ordering::Equal);
    }

    #[test]
    fn db_key_values_insert() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key", "value").into())
            .unwrap();

        assert_eq!(kvs.len(), 4);
        assert_eq!(
            kvs.values(&storage, 3).unwrap(),
            vec![("key", "value").into()]
        );
    }

    #[test]
    fn db_key_values_insert_or_replace() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key", "value").into())
            .unwrap();
        let old = kvs
            .insert_or_replace(&mut storage, 3, &("key", "value1").into())
            .unwrap();

        assert_eq!(kvs.len(), 4);
        assert_eq!(
            kvs.values(&storage, 3).unwrap(),
            vec![("key", "value1").into()]
        );
        assert_eq!(old, Some(("key", "value").into()));
    }

    #[test]
    fn db_key_values_insert_multiple() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key1", "value1").into())
            .unwrap();
        kvs.insert_value(&mut storage, 3, &("key2", "value2").into())
            .unwrap();

        assert_eq!(kvs.len(), 4);
        assert_eq!(
            kvs.values(&storage, 3).unwrap(),
            vec![("key1", "value1").into(), ("key2", "value2").into()]
        );
    }

    #[test]
    fn db_key_values_insert_to_default_initialized() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key", "value").into())
            .unwrap();
        kvs.insert_value(&mut storage, 1, &("key1", "value1").into())
            .unwrap();

        assert_eq!(kvs.len(), 4);
        assert_eq!(
            kvs.values(&storage, 1).unwrap(),
            vec![("key1", "value1").into()]
        );
    }

    #[test]
    fn db_key_values_remove_all_values() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key", "value").into())
            .unwrap();
        kvs.insert_value(&mut storage, 1, &("key1", "value1").into())
            .unwrap();
        kvs.remove(&mut storage, 1).unwrap();

        assert_eq!(kvs.len(), 4);
        assert_eq!(kvs.values(&storage, 1).unwrap(), vec![]);
    }

    #[test]
    fn db_key_values_remove_key_value() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key1", "value1").into())
            .unwrap();
        kvs.insert_value(&mut storage, 3, &("key2", "value2").into())
            .unwrap();
        kvs.insert_value(&mut storage, 3, &("key3", "value3").into())
            .unwrap();
        kvs.remove_value(&mut storage, 3, &"key2".into()).unwrap();
        kvs.remove_value(&mut storage, 3, &"key10".into()).unwrap();
        kvs.remove_value(&mut storage, 10, &"key10".into()).unwrap();

        assert_eq!(kvs.len(), 4);
        assert_eq!(
            kvs.values(&storage, 3).unwrap(),
            vec![("key1", "value1").into(), ("key3", "value3").into()]
        );
    }

    #[test]
    fn db_key_values_keys() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key1", "value1").into())
            .unwrap();
        kvs.insert_value(&mut storage, 3, &("key2", "value2").into())
            .unwrap();
        kvs.insert_value(&mut storage, 3, &("key3", "value3").into())
            .unwrap();

        assert_eq!(
            kvs.keys(&storage, 3).unwrap(),
            vec!["key1".into(), "key2".into(), "key3".into()]
        );
        assert_eq!(kvs.key_count(&storage, 3).unwrap(), 3);
        assert_eq!(kvs.keys(&storage, 1).unwrap(), vec![]);
        assert_eq!(kvs.key_count(&storage, 1).unwrap(), 0);
    }

    #[test]
    fn db_key_values_value() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key1", "value1").into())
            .unwrap();
        kvs.insert_value(&mut storage, 3, &("key2", "value2").into())
            .unwrap();
        kvs.insert_value(&mut storage, 3, &("key3", "value3").into())
            .unwrap();

        assert_eq!(
            kvs.value(&storage, 3, &"key1".into()).unwrap(),
            Some("value1".into())
        );
        assert_eq!(
            kvs.value(&storage, 3, &"key2".into()).unwrap(),
            Some("value2".into())
        );
        assert_eq!(
            kvs.value(&storage, 3, &"key3".into()).unwrap(),
            Some("value3".into())
        );
        assert_eq!(kvs.value(&storage, 3, &"key4".into()).unwrap(), None);
    }

    #[test]
    fn db_key_values_values() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key1", "value1").into())
            .unwrap();

        assert_eq!(kvs.values(&storage, 1).unwrap(), vec![]);
        assert_eq!(
            kvs.values(&storage, 3).unwrap(),
            vec![("key1", "value1").into()]
        );
    }

    #[test]
    fn db_key_values_values_by_keys() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
        kvs.insert_value(&mut storage, 3, &("key1", "value1").into())
            .unwrap();
        kvs.insert_value(&mut storage, 3, &("key2", "value2").into())
            .unwrap();
        kvs.insert_value(&mut storage, 3, &("key3", "value3").into())
            .unwrap();

        assert_eq!(
            kvs.values_by_keys(&storage, 3, &["key3".into(), "key1".into()])
                .unwrap(),
            vec![("key3", "value3").into(), ("key1", "value1").into()]
        );
        assert_eq!(
            kvs.values_by_keys(&storage, 3, &["key4".into()]).unwrap(),
            vec![]
        );
        assert_eq!(
            kvs.values_by_keys(&storage, 1, &["key4".into()]).unwrap(),
            vec![]
        );
    }

    #[test]
    fn db_key_values_from_storage() {
        let mut storage = Storage::<MemoryStorage>::new("test").unwrap();
        let index;
        {
            let mut kvs = DbKeyValues(DbVec::new(&mut storage).unwrap());
            kvs.insert_value(&mut storage, 3, &("key1", "value1").into())
                .unwrap();
            kvs.insert_value(&mut storage, 3, &("key2", "value2").into())
                .unwrap();
            kvs.insert_value(&mut storage, 3, &("key3", "value3").into())
                .unwrap();
            index = kvs.storage_index();
        }
        let kvs = DbKeyValues::from_storage(&storage, index).unwrap();
        assert_eq!(kvs.len(), 4);
    }
}
