use super::dictionary::dictionary_data_storage::DictionaryDataStorage;
use super::dictionary::dictionary_impl::DictionaryImpl;
use super::dictionary::Dictionary;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub type DictionaryStorage<T, Data = FileStorage> =
    DictionaryImpl<T, DictionaryDataStorage<T, Data>>;

#[allow(dead_code)]
impl<T, Data> DictionaryStorage<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + StorageValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        Ok(Self {
            data: DictionaryDataStorage::new(storage)?,
            phantom_data: PhantomData,
        })
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: &StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            data: DictionaryDataStorage::from_storage(storage, index)?,
            phantom_data: PhantomData,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index()
    }

    pub fn to_dictionary(&self) -> Result<Dictionary<T>, DbError> {
        Ok(Dictionary {
            data: self.data.to_dictionary_data_memory()?,
            phantom_data: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::collided_value::CollidedValue;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn count_invalid_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        assert_eq!(dictionary.count(u64::MAX), Ok(None));
    }

    #[test]
    fn index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.index(&10), Ok(Some(index)));
    }

    #[test]
    fn index_missing_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        assert_eq!(dictionary.index(&10), Ok(None));
    }

    #[test]
    fn index_removed_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.index(&10), Ok(None));
    }

    #[test]
    fn index_reuse() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index1 = dictionary.insert(&5).unwrap();
        let index2 = dictionary.insert(&10).unwrap();
        let index3 = dictionary.insert(&7).unwrap();

        dictionary.remove(index2).unwrap();
        dictionary.remove(index1).unwrap();
        dictionary.remove(index3).unwrap();

        assert_eq!(dictionary.count(index1), Ok(None));
        assert_eq!(dictionary.count(index2), Ok(None));
        assert_eq!(dictionary.count(index3), Ok(None));

        assert_eq!(dictionary.insert(&3), Ok(index3.clone()));
        assert_eq!(dictionary.insert(&2), Ok(index1.clone()));
        assert_eq!(dictionary.insert(&1), Ok(index2.clone()));

        assert_eq!(dictionary.value(index1), Ok(Some(2)));
        assert_eq!(dictionary.value(index2), Ok(Some(1)));
        assert_eq!(dictionary.value(index3), Ok(Some(3)));
    }

    #[test]
    fn index_with_collisions() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<CollidedValue<i64>>::new(storage).unwrap();

        let index1 = dictionary.insert(&CollidedValue::new(1)).unwrap();
        let index2 = dictionary.insert(&CollidedValue::new(2)).unwrap();
        let index3 = dictionary.insert(&CollidedValue::new(3)).unwrap();

        assert_eq!(dictionary.index(&CollidedValue::new(1)), Ok(Some(index1)));
        assert_eq!(dictionary.index(&CollidedValue::new(2)), Ok(Some(index2)));
        assert_eq!(dictionary.index(&CollidedValue::new(3)), Ok(Some(index3)));
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));
        assert_eq!(dictionary.value(index), Ok(Some(10_i64)));
        assert_eq!(dictionary.count(index), Ok(Some(1)));
    }

    #[test]
    fn insert_multiple() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index1 = dictionary.insert(&10).unwrap();
        let index2 = dictionary.insert(&15).unwrap();
        let index3 = dictionary.insert(&20).unwrap();

        assert_eq!(dictionary.len(), Ok(3));

        assert_eq!(dictionary.value(index1).unwrap(), Some(10_i64));
        assert_eq!(dictionary.count(index1), Ok(Some(1)));

        assert_eq!(dictionary.value(index2).unwrap(), Some(15_i64));
        assert_eq!(dictionary.count(index2), Ok(Some(1)));

        assert_eq!(dictionary.value(index3).unwrap(), Some(20_i64));
        assert_eq!(dictionary.count(index3), Ok(Some(1)));
    }

    #[test]
    fn insert_same() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        dictionary.insert(&10).unwrap();

        let index2 = dictionary.insert(&15).unwrap();

        assert_eq!(dictionary.insert(&15).unwrap(), index2);
        assert_eq!(dictionary.insert(&15).unwrap(), index2);

        dictionary.insert(&20).unwrap();

        assert_eq!(dictionary.len(), Ok(3));
        assert_eq!(dictionary.count(index2), Ok(Some(3)));
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(None));
        assert_eq!(dictionary.count(index), Ok(None));
    }

    #[test]
    fn remove_duplicated() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();
        dictionary.insert(&10).unwrap();
        dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.value(index), Ok(Some(10)));
        assert_eq!(dictionary.count(index), Ok(Some(3)));

        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(Some(10)));
        assert_eq!(dictionary.count(index), Ok(Some(2)));

        dictionary.remove(index).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(None));
        assert_eq!(dictionary.count(index), Ok(None));
    }

    #[test]
    fn remove_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));

        dictionary.remove(index + 1).unwrap();

        assert_eq!(dictionary.len(), Ok(1));
    }

    #[test]
    fn restore_from_file() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let storage_index;
        let index1;
        let index2;
        let index3;
        let index4;

        {
            let mut dictionary = DictionaryStorage::<i64>::new(storage.clone()).unwrap();
            storage_index = dictionary.storage_index();

            index1 = dictionary.insert(&10).unwrap();
            dictionary.insert(&10).unwrap();
            index2 = dictionary.insert(&15).unwrap();
            index3 = dictionary.insert(&7).unwrap();
            index4 = dictionary.insert(&20).unwrap();
            dictionary.remove(index2).unwrap();
        }

        let dictionary = DictionaryStorage::<i64>::from_storage(storage, &storage_index).unwrap();

        assert_eq!(dictionary.len(), Ok(3));
        assert_eq!(dictionary.count(index1), Ok(Some(2)));
        assert_eq!(dictionary.value(index1), Ok(Some(10)));
        assert_eq!(dictionary.value(index2), Ok(None));
        assert_eq!(dictionary.value(index3), Ok(Some(7)));
        assert_eq!(dictionary.value(index4), Ok(Some(20)));
    }

    #[test]
    fn value_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let dictionary = DictionaryStorage::<i64>::new(storage).unwrap();
        assert_eq!(dictionary.value(1), Ok(None));
    }
}
