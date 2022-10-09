use agdb_storage::StorageFile;
use agdb_storage_map::StorageMap;
use agdb_test_file::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn remove() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(7, 20).unwrap();

    assert_eq!(map.count(), 3);
    map.remove(&5).unwrap();

    assert_eq!(map.count(), 2);
    assert_eq!(map.value(&1), Ok(Some(10)));
    assert_eq!(map.value(&5), Ok(None));
    assert_eq!(map.value(&7), Ok(Some(20)));
}

#[test]
fn remove_deleted() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(7, 20).unwrap();

    assert_eq!(map.count(), 3);

    map.remove(&5).unwrap();

    assert_eq!(map.count(), 2);
    assert_eq!(map.value(&5), Ok(None));

    map.remove(&5).unwrap();

    assert_eq!(map.count(), 2);
}

#[test]
fn remove_missing() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

    assert_eq!(map.count(), 0);
    assert_eq!(map.remove(&0), Ok(()));
    assert_eq!(map.count(), 0);
}

#[test]
fn remove_shrinks_capacity() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

    for i in 0..100 {
        map.insert(i, i).unwrap();
    }

    assert_eq!(map.count(), 100);
    assert_eq!(map.capacity(), 128);

    for i in 0..100 {
        map.remove(&i).unwrap();
    }

    assert_eq!(map.count(), 0);
    assert_eq!(map.capacity(), 64);
}
