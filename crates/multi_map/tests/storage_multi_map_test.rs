use agdb_db_error::DbError;
use agdb_multi_map::StorageMultiMap;
use agdb_storage::StorageFile;
use agdb_storage::StorageIndex;
use agdb_test_file::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn iter() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(7, 20).unwrap();
    map.insert(2, 30).unwrap();
    map.insert(2, 50).unwrap();
    map.insert(4, 13).unwrap();
    map.remove_key(&7).unwrap();

    let mut actual = map.iter().collect::<Vec<(i64, i64)>>();
    actual.sort();
    let expected: Vec<(i64, i64)> = vec![(1, 10), (2, 30), (2, 50), (4, 13), (5, 15), (5, 15)];

    assert_eq!(actual, expected);
}

#[test]
fn to_multi_map() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(7, 20).unwrap();
    map.remove_key(&5).unwrap();

    let other = map.to_multi_map().unwrap();

    assert_eq!(other.count(), 2);
    assert_eq!(other.value(&1).unwrap(), Some(10));
    assert_eq!(other.value(&5).unwrap(), None);
    assert_eq!(other.value(&7).unwrap(), Some(20));
}

#[test]
fn to_multi_map_empty() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();
    let other = map.to_multi_map().unwrap();

    assert_eq!(other.count(), 0);
}

#[test]
fn try_from_storage_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let index;

    {
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage.clone()).unwrap();
        map.insert(1, 1).unwrap();
        map.insert(3, 2).unwrap();
        map.insert(3, 3).unwrap();
        map.insert(5, 3).unwrap();
        map.remove_key(&1).unwrap();
        index = map.storage_index();
    }

    let map = StorageMultiMap::<i64, i64>::try_from((storage, index)).unwrap();

    assert_eq!(
        map.iter().collect::<Vec<(i64, i64)>>(),
        vec![(3_i64, 2_i64), (3_i64, 3_i64), (5_i64, 3_i64)]
    );
}

#[test]
fn try_from_storage_missing_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    assert_eq!(
        StorageMultiMap::<i64, i64>::try_from((storage, StorageIndex::from(1_i64)))
            .err()
            .unwrap(),
        DbError::from("index '1' not found")
    );
}
