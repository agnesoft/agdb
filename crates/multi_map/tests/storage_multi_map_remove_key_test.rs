use agdb_multi_map::StorageMultiMap;
use agdb_storage::StorageFile;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn remove_deleted_key() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(7, 20).unwrap();

    assert_eq!(map.count(), 3);

    map.remove_key(&5).unwrap();

    assert_eq!(map.count(), 2);
    assert_eq!(map.value(&5), Ok(None));

    map.remove_key(&5).unwrap();

    assert_eq!(map.count(), 2);
}

#[test]
fn remove_key() {
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
    map.remove_key(&5).unwrap();

    assert_eq!(map.count(), 1);
    assert_eq!(map.value(&1), Ok(Some(7)));
    assert_eq!(map.values(&5), Ok(Vec::<i64>::new()));
}

#[test]
fn remove_missing_key() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    assert_eq!(map.count(), 0);

    map.remove_key(&5).unwrap();

    assert_eq!(map.count(), 0);
}

#[test]
fn remove_shrinks_capacity() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    for i in 0..100 {
        map.insert(i, i).unwrap();
    }

    assert_eq!(map.count(), 100);
    assert_eq!(map.capacity(), 128);

    for i in 0..100 {
        map.remove_key(&i).unwrap();
    }

    assert_eq!(map.count(), 0);
    assert_eq!(map.capacity(), 64);
}
