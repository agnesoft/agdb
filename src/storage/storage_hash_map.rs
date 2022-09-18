use super::file_storage::FileStorage;
use super::serialize::Serialize;
use super::stable_hash::StableHash;
use super::storage_hash_map_key_value::StorageHashMapKeyValue;
use super::Storage;
use crate::DbError;

#[allow(dead_code)]
pub(crate) struct HashMap<K: Serialize + StableHash, T: Serialize, S: Storage = FileStorage> {
    storage: std::rc::Rc<std::cell::RefCell<S>>,
    storage_index: i64,
    size: u64,
    capacity: u64,
    phantom_data: std::marker::PhantomData<(K, T)>,
}

#[allow(dead_code)]
impl<K: Serialize + StableHash, T: Serialize, S: Storage> HashMap<K, T, S> {
    pub(crate) fn insert(&mut self, key: K, value: T) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let pos = hash % self.capacity;

        self.storage.borrow_mut().transaction();

        let offset = self.free_offset(pos)?;
        self.storage.borrow_mut().insert_at(
            self.storage_index,
            offset,
            &StorageHashMapKeyValue { key, value },
        )?;

        self.storage.borrow_mut().commit()?;

        Ok(())
    }

    pub(crate) fn value(&mut self, key: &K) -> Result<Option<T>, DbError> {
        if let Some(offset) = self.find_value_offset(key)? {
            return Ok(Some(
                self.storage
                    .borrow_mut()
                    .value_at::<T>(self.storage_index, offset)?,
            ));
        }

        Ok(None)
    }

    fn find_value_offset(&mut self, key: &K) -> Result<Option<u64>, DbError> {
        Ok(None)
    }

    fn free_offset(&mut self, pos: u64) -> Result<u64, DbError> {
        Ok(0)
    }
}

impl<K: Serialize + StableHash, T: Serialize, S: Storage>
    TryFrom<std::rc::Rc<std::cell::RefCell<S>>> for HashMap<K, T, S>
{
    type Error = DbError;

    fn try_from(storage: std::rc::Rc<std::cell::RefCell<S>>) -> Result<Self, Self::Error> {
        let index = storage.borrow_mut().insert(&0_u64)?;

        Ok(Self {
            storage,
            storage_index: index,
            size: 0,
            capacity: 0,
            phantom_data: std::marker::PhantomData::<(K, T)>,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn insert() {
        let test_file = TestFile::from("./storage_hash_map-insert.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = HashMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();

        assert_eq!(map.value(&1), Ok(Some(10)));
    }
}
