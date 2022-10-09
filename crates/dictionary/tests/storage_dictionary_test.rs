use agdb_dictionary::DictionaryIndex;
use agdb_dictionary::StorageDictionary;
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn count_invalid_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

    assert_eq!(dictionary.count(&DictionaryIndex::from(-1_i64)), Ok(None));
}

#[test]
fn restore_from_file() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
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
        dictionary.remove(&index2).unwrap();
    }

    let dictionary = StorageDictionary::<i64>::try_from((storage, storage_index)).unwrap();

    assert_eq!(dictionary.len(), Ok(3));
    assert_eq!(dictionary.count(&index1), Ok(Some(2)));
    assert_eq!(dictionary.value(&index1), Ok(Some(10)));
    assert_eq!(dictionary.value(&index2), Ok(None));
    assert_eq!(dictionary.value(&index3), Ok(Some(7)));
    assert_eq!(dictionary.value(&index4), Ok(Some(20)));
}

#[test]
fn value_missing_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();
    assert_eq!(dictionary.value(&DictionaryIndex::from(1_i64)), Ok(None));
}
