use agdb_dictionary::StorageDictionary;
use agdb_storage::StorageFile;
use agdb_test_utilities::CollidedValue;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

    let index = dictionary.insert(&10).unwrap();

    assert_eq!(dictionary.index(&10), Ok(Some(index)));
}

#[test]
fn index_missing_value() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

    assert_eq!(dictionary.index(&10), Ok(None));
}

#[test]
fn index_removed_value() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

    let index = dictionary.insert(&10).unwrap();
    dictionary.remove(index).unwrap();

    assert_eq!(dictionary.index(&10), Ok(None));
}

#[test]
fn index_reuse() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
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
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut dictionary = StorageDictionary::<CollidedValue<i64>>::try_from(storage).unwrap();

    let index1 = dictionary.insert(&CollidedValue::new(1)).unwrap();
    let index2 = dictionary.insert(&CollidedValue::new(2)).unwrap();
    let index3 = dictionary.insert(&CollidedValue::new(3)).unwrap();

    assert_eq!(dictionary.index(&CollidedValue::new(1)), Ok(Some(index1)));
    assert_eq!(dictionary.index(&CollidedValue::new(2)), Ok(Some(index2)));
    assert_eq!(dictionary.index(&CollidedValue::new(3)), Ok(Some(index3)));
}

#[test]
fn value_missing_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let dictionary = StorageDictionary::<i64>::try_from(storage).unwrap();

    assert_eq!(dictionary.value(1), Ok(None));
}
