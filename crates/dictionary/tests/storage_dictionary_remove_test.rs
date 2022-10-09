use agdb_dictionary::DictionaryIndex;
use agdb_dictionary::StorageDictionary;
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn remove() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

    let index = dictionary.insert(&10).unwrap();
    dictionary.remove(&index).unwrap();

    assert_eq!(dictionary.value(&index), Ok(None));
    assert_eq!(dictionary.count(&index), Ok(None));
}

#[test]
fn remove_duplicated() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

    let index = dictionary.insert(&10).unwrap();
    dictionary.insert(&10).unwrap();
    dictionary.insert(&10).unwrap();

    assert_eq!(dictionary.value(&index), Ok(Some(10)));
    assert_eq!(dictionary.count(&index), Ok(Some(3)));

    dictionary.remove(&index).unwrap();

    assert_eq!(dictionary.value(&index), Ok(Some(10)));
    assert_eq!(dictionary.count(&index), Ok(Some(2)));

    dictionary.remove(&index).unwrap();
    dictionary.remove(&index).unwrap();

    assert_eq!(dictionary.value(&index), Ok(None));
    assert_eq!(dictionary.count(&index), Ok(None));
}

#[test]
fn remove_missing() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

    let index = dictionary.insert(&10).unwrap();

    assert_eq!(dictionary.len(), Ok(1));

    dictionary
        .remove(&DictionaryIndex::from(index.value() + 1))
        .unwrap();

    assert_eq!(dictionary.len(), Ok(1));
}
