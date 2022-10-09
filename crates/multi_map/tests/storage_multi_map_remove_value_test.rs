use agdb_multi_map::StorageMultiMap;
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn remove_value() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(1, 7).unwrap();
    map.insert(5, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(5, 20).unwrap();

    assert_eq!(map.count(), 4);
    map.remove_value(&5, &15).unwrap();

    assert_eq!(map.count(), 3);
    assert_eq!(map.value(&1), Ok(Some(7)));
    assert_eq!(map.values(&5), Ok(vec![10_i64, 20_i64]));
}

#[test]
fn remove_missing_value() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    map.remove_value(&5, &10).unwrap();

    assert_eq!(map.count(), 0);
}
