use super::file_storage::FileStorage;
use super::serialize::Serialize;
use super::stable_hash::StableHash;
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

impl<K: Serialize + StableHash, T: Serialize, S: Storage> HashMap<K, T, S> {
    pub(crate) fn insert(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let pos = hash % self.capacity;
        let offset = self.free_offset(pos);
        self.storage.borrow_mut().transaction();
        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, offset, value)?;

        self.storage.borrow_mut().commit()?;

        Ok(())
    }

    pub(crate) fn value(&mut self, key: &K) -> Result<Option<T>, DbError> {
        todo!()
    }

    fn free_offset(&mut self, pos: u64) -> u64 {
        0
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

        map.insert(&1, &10).unwrap();

        assert_eq!(map.value(&1), Ok(Some(10)));
    }
}
