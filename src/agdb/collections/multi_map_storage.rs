use super::map::map_data_storage::MapDataStorage;
use super::map::multi_map_impl::MultiMapImpl;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

pub type MultiMapStorage<K, T, Data = FileStorage> = MultiMapImpl<K, T, MapDataStorage<K, T, Data>>;

#[allow(dead_code)]
impl<K, T, Data> MultiMapStorage<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash + StorageValue,
    T: Default + Eq + PartialEq + StorageValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        Ok(Self {
            data: MapDataStorage::<K, T, Data>::new(storage)?,
            phantom_marker: PhantomData,
        })
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: &StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            data: MapDataStorage::<K, T, Data>::from_storage(storage, index)?,
            phantom_marker: PhantomData,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn new() {
        todo!()
    }

    #[test]
    fn from_storage() {
        todo!()
    }

    #[test]
    fn storage_index() {
        todo!()
    }
}
