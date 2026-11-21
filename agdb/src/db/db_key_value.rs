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
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
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

        kvs.reserve(storage, kvs.len() + 1)?;
        kvs.push(storage, value)
    }

    pub fn insert_or_replace(
        &mut self,
        storage: &mut Storage<S>,
        index: u64,
        value: &DbKeyValue,
    ) -> Result<Option<DbKeyValue>, DbError> {
        let Some(mut kvs) = self.existing_kvs(storage, index)? else {
            self.insert_value(storage, index, value)?;
            return Ok(None);
        };

        if let Some((index, kv)) = find(&kvs, storage, &value.key)? {
            kvs.replace(storage, index, value)?;
            Ok(Some(kv))
        } else {
            kvs.reserve(storage, kvs.len() + 1)?;
            kvs.push(storage, value)?;
            Ok(None)
        }
    }

    pub fn key_count(&self, storage: &Storage<S>, index: u64) -> Result<u64, DbError> {
        Ok(self
            .existing_kvs(storage, index)?
            .map_or(0, |kvs| kvs.len()))
    }

    pub fn keys(&self, storage: &Storage<S>, index: u64) -> Result<Vec<DbValue>, DbError> {
        let Some(kvs) = self.existing_kvs(storage, index)? else {
            return Ok(vec![]);
        };
        kvs.iter(storage).map(|kv| Ok(kv?.key)).collect()
    }

    pub fn remove(&mut self, storage: &mut Storage<S>, index: u64) -> Result<(), DbError> {
        let Some(kvs) = self.existing_kvs(storage, index)? else {
            return Ok(());
        };
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
        let Some(mut kvs) = self.existing_kvs(storage, index)? else {
            return Ok(());
        };

        if let Some((index, _)) = find(&kvs, storage, key)? {
            kvs.remove(storage, index)?;
        }

        Ok(())
    }

    pub fn reserve_capacity(
        &mut self,
        storage: &mut Storage<S>,
        index: u64,
        len: u64,
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

        kvs.reserve(storage, len)
    }

    pub fn shrink_to_fit(&mut self, storage: &mut Storage<S>) -> Result<(), DbError> {
        for index in 0..self.0.len() {
            if let Some(mut kvs) = self.existing_kvs(storage, index)? {
                kvs.shrink_to_fit(storage)?;
            }
        }

        self.0.shrink_to_fit(storage)
    }

    pub fn value(
        &self,
        storage: &Storage<S>,
        index: u64,
        key: &DbValue,
    ) -> Result<Option<DbValue>, DbError> {
        let Some(kvs) = self.existing_kvs(storage, index)? else {
            return Ok(None);
        };
        Ok(find(&kvs, storage, key)?.map(|found| found.1.value))
    }

    pub fn values(&self, storage: &Storage<S>, index: u64) -> Result<Vec<DbKeyValue>, DbError> {
        let Some(kvs) = self.existing_kvs(storage, index)? else {
            return Ok(vec![]);
        };
        kvs.iter(storage).collect()
    }

    pub fn values_by_keys(
        &self,
        storage: &Storage<S>,
        index: u64,
        keys: &[DbValue],
    ) -> Result<Vec<DbKeyValue>, DbError> {
        let Some(kvs) = self.existing_kvs(storage, index)? else {
            return Ok(vec![]);
        };
        let mut values = vec![];

        for (_, key_index, value_index) in stored(&kvs, storage)? {
            if let Some(pos) = position(keys, key_index, storage)? {
                let value = DbValue::load_db_value(value_index, storage)?;
                values.push((
                    pos,
                    DbKeyValue {
                        key: keys[pos].clone(),
                        value,
                    },
                ));
            }
        }

        values.sort_by_key(|(i, _)| *i);
        Ok(values.into_iter().map(|(_, kv)| kv).collect())
    }

    fn existing_kvs(
        &self,
        storage: &Storage<S>,
        index: u64,
    ) -> Result<Option<DbVec<DbKeyValue, S>>, DbError> {
        if self.0.len() <= index {
            return Ok(None);
        }

        let storage_index = self.0.value(storage, index)?;

        if storage_index.0 == 0 {
            return Ok(None);
        }

        Ok(Some(DbVec::from_storage(storage, storage_index)?))
    }
}

fn find<S: StorageData>(
    kvs: &DbVec<DbKeyValue, S>,
    storage: &Storage<S>,
    key: &DbValue,
) -> Result<Option<(u64, DbKeyValue)>, DbError> {
    for (index, key_index, value_index) in stored(kvs, storage)? {
        if key.equals_stored(key_index, storage)? {
            let value = DbValue::load_db_value(value_index, storage)?;
            return Ok(Some((
                index,
                DbKeyValue {
                    key: key.clone(),
                    value,
                },
            )));
        }
    }

    Ok(None)
}

fn position<S: StorageData>(
    keys: &[DbValue],
    key_index: DbValueIndex,
    storage: &Storage<S>,
) -> Result<Option<usize>, DbError> {
    for (pos, key) in keys.iter().enumerate() {
        if key.equals_stored(key_index, storage)? {
            return Ok(Some(pos));
        }
    }

    Ok(None)
}

/// Reads the key and value indexes of all `kvs` at once so that
/// the values can be loaded only for the matching keys.
fn stored<S: StorageData>(
    kvs: &DbVec<DbKeyValue, S>,
    storage: &Storage<S>,
) -> Result<Vec<(u64, DbValueIndex, DbValueIndex)>, DbError> {
    let bytes = kvs.bytes_at(storage, 0, kvs.len())?;
    let mut indexes = Vec::with_capacity(kvs.len() as usize);

    for (index, kv) in bytes
        .as_chunks::<16>()
        .0
        .as_chunks::<2>()
        .0
        .iter()
        .enumerate()
    {
        indexes.push((
            index as u64,
            DbValueIndex::from_data(kv[0]),
            DbValueIndex::from_data(kv[1]),
        ));
    }

    Ok(indexes)
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
