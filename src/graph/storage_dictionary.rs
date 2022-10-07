use super::dictionary_data_storage::DictionaryDataStorage;
use super::dictionary_data_storage_indexes::DictionaryDataStorageIndexes;
use super::dictionary_impl::DictionaryImpl;
use super::dictionary_value::DictionaryValue;
use crate::storage::StableHash;
use crate::storage::StorageHashMultiMap;
use crate::storage::StorageVec;
use crate::DbError;
use serialize::Serialize;
use storage::FileStorage;
use storage::Storage;

pub(crate) type StorageDictionary<T, Data = FileStorage> =
    DictionaryImpl<T, DictionaryDataStorage<T, Data>>;

#[allow(dead_code)]
impl<T, Data> StorageDictionary<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: Storage,
{
    pub(crate) fn storage_index(&self) -> i64 {
        self.data.storage_index
    }
}

impl<T, Data> TryFrom<std::rc::Rc<std::cell::RefCell<Data>>> for StorageDictionary<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: std::rc::Rc<std::cell::RefCell<Data>>) -> Result<Self, Self::Error> {
        let index = StorageHashMultiMap::<u64, i64, Data>::try_from(storage.clone())?;
        let mut values = StorageVec::<DictionaryValue<T>, Data>::try_from(storage.clone())?;
        values.push(&DictionaryValue::default())?;

        let storage_index = storage.borrow_mut().insert(&DictionaryDataStorageIndexes {
            index: index.storage_index(),
            values: values.storage_index(),
        })?;

        Ok(StorageDictionary::<T, Data> {
            data: DictionaryDataStorage::<T, Data> {
                storage,
                storage_index,
                index,
                values,
            },
            phantom_data: std::marker::PhantomData,
        })
    }
}

impl<T, Data> TryFrom<(std::rc::Rc<std::cell::RefCell<Data>>, i64)> for StorageDictionary<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (std::rc::Rc<std::cell::RefCell<Data>>, i64),
    ) -> Result<Self, Self::Error> {
        let indexes = storage_with_index
            .0
            .borrow_mut()
            .value::<DictionaryDataStorageIndexes>(storage_with_index.1)?;
        let index = StorageHashMultiMap::<u64, i64, Data>::try_from((
            storage_with_index.0.clone(),
            indexes.index,
        ))?;
        let values = StorageVec::<DictionaryValue<T>, Data>::try_from((
            storage_with_index.0.clone(),
            indexes.values,
        ))?;

        Ok(StorageDictionary::<T, Data> {
            data: DictionaryDataStorage::<T, Data> {
                storage: storage_with_index.0,
                storage_index: storage_with_index.1,
                index,
                values,
            },
            phantom_data: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage::FileStorage;
    use test_file::TestFile;

    #[derive(Clone, Default, Eq, PartialEq)]
    struct CollidedValue {
        pub(super) value: i64,
    }

    impl Serialize for CollidedValue {
        fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
            Ok(CollidedValue {
                value: i64::deserialize(bytes)?,
            })
        }

        fn serialize(&self) -> Vec<u8> {
            self.value.serialize()
        }
    }

    impl StableHash for CollidedValue {
        fn stable_hash(&self) -> u64 {
            1
        }
    }

    #[test]
    fn collided_value() {
        let value = CollidedValue { value: 10 };
        let bytes = value.serialize();
        let other = CollidedValue::deserialize(&bytes).unwrap();

        assert!(value == other);
    }

    #[test]
    fn count_invalid_index() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

        assert_eq!(dictionary.count(-1), Ok(None));
    }

    #[test]
    fn index() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.index(&10), Ok(Some(index)));
    }

    #[test]
    fn index_missing_value() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

        assert_eq!(dictionary.index(&10), Ok(None));
    }

    #[test]
    fn index_removed_value() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.index(&10), Ok(None));
    }

    #[test]
    fn index_reuse() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

        let index1 = dictionary.insert(&5).unwrap();
        let index2 = dictionary.insert(&10).unwrap();
        let index3 = dictionary.insert(&7).unwrap();

        dictionary.remove(index2).unwrap();
        dictionary.remove(index1).unwrap();
        dictionary.remove(index3).unwrap();

        assert_eq!(dictionary.count(index1), Ok(None));
        assert_eq!(dictionary.count(index2), Ok(None));
        assert_eq!(dictionary.count(index3), Ok(None));

        assert_eq!(dictionary.insert(&3), Ok(index3));
        assert_eq!(dictionary.insert(&2), Ok(index1));
        assert_eq!(dictionary.insert(&1), Ok(index2));

        assert_eq!(dictionary.value(index1), Ok(Some(2)));
        assert_eq!(dictionary.value(index2), Ok(Some(1)));
        assert_eq!(dictionary.value(index3), Ok(Some(3)));
    }

    #[test]
    fn index_with_collisions() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<CollidedValue>::try_from(storage).unwrap();

        let index1 = dictionary.insert(&CollidedValue { value: 1 }).unwrap();
        let index2 = dictionary.insert(&CollidedValue { value: 2 }).unwrap();
        let index3 = dictionary.insert(&CollidedValue { value: 3 }).unwrap();

        assert_eq!(
            dictionary.index(&CollidedValue { value: 1 }),
            Ok(Some(index1))
        );

        assert_eq!(
            dictionary.index(&CollidedValue { value: 2 }),
            Ok(Some(index2))
        );

        assert_eq!(
            dictionary.index(&CollidedValue { value: 3 }),
            Ok(Some(index3))
        );
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));
        assert_eq!(dictionary.value(index), Ok(Some(10_i64)));
        assert_eq!(dictionary.count(index), Ok(Some(1)));
    }

    #[test]
    fn insert_multiple() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

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
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

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
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(None));
        assert_eq!(dictionary.count(index), Ok(None));
    }

    #[test]
    fn remove_duplicated() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

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
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));

        dictionary.remove(index + 1).unwrap();

        assert_eq!(dictionary.len(), Ok(1));
    }

    #[test]
    fn restore_from_file() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let storage_index;
        let index1;
        let index2;
        let index3;
        let index4;

        {
            let mut dictionary = StorageDictionary::<i64>::try_from(storage.clone()).unwrap();
            storage_index = dictionary.storage_index();

            index1 = dictionary.insert(&10).unwrap();
            dictionary.insert(&10).unwrap();
            index2 = dictionary.insert(&15).unwrap();
            index3 = dictionary.insert(&7).unwrap();
            index4 = dictionary.insert(&20).unwrap();
            dictionary.remove(index2).unwrap();
        }

        let dictionary = StorageDictionary::<i64>::try_from((storage, storage_index)).unwrap();

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
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

        assert_eq!(dictionary.value(1), Ok(None));
    }
}
