use agdb_multi_map::StorageMultiMap;
use agdb_storage::StorageFile;
use agdb_test_file::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn insert() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(7, 20).unwrap();

    assert_eq!(map.count(), 3);
    assert_eq!(map.value(&1), Ok(Some(10)));
    assert_eq!(map.value(&5), Ok(Some(15)));
    assert_eq!(map.value(&7), Ok(Some(20)));
}

#[test]
fn insert_reallocate() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    assert_eq!(map.capacity(), 1);

    for i in 0..100 {
        map.insert(i, i).unwrap();
    }

    assert_eq!(map.count(), 100);
    assert_eq!(map.capacity(), 128);

    for i in 0..100 {
        assert_eq!(map.value(&i), Ok(Some(i)));
    }
}

#[test]
fn insert_reallocate_with_collisions() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    for i in 0..50 {
        map.insert(i * 64, i).unwrap();
        map.insert(i * 64, i + 1).unwrap();
    }

    for i in 0..50 {
        assert_eq!(map.value(&(i * 64)), Ok(Some(i)));
    }
}

#[test]
fn insert_same_key() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    assert_eq!(map.count(), 2);
    map.insert(5, 20).unwrap();
    assert_eq!(map.count(), 3);

    assert_eq!(map.value(&1), Ok(Some(10)));
    assert_eq!(map.value(&5), Ok(Some(15)));
}
