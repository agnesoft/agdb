use agdb_storage::StorageFile;
use agdb_storage_map::StorageMap;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn reserve_larger() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();
    map.insert(1, 1).unwrap();

    let capacity = map.capacity() + 10;
    let size = map.count();

    map.reserve(capacity).unwrap();

    assert_eq!(map.capacity(), capacity);
    assert_eq!(map.count(), size);
    assert_eq!(map.value(&1), Ok(Some(1)));
}

#[test]
fn reserve_same() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();
    map.insert(1, 1).unwrap();

    let capacity = map.capacity();
    let size = map.count();

    map.reserve(capacity).unwrap();

    assert_eq!(map.capacity(), capacity);
    assert_eq!(map.count(), size);
}

#[test]
fn reserve_smaller() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();
    map.insert(1, 1).unwrap();

    let current_capacity = map.capacity();
    let capacity = current_capacity - 10;
    let size = map.count();

    map.reserve(capacity).unwrap();

    assert_eq!(map.capacity(), current_capacity);
    assert_eq!(map.count(), size);
}
