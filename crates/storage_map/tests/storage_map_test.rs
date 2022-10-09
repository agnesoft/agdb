use agdb_db_error::DbError;
use agdb_storage::StorageFile;
use agdb_storage::StorageIndex;
use agdb_storage_map::StorageMap;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn iter() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(7, 20).unwrap();
    map.insert(2, 30).unwrap();
    map.insert(4, 13).unwrap();
    map.remove(&7).unwrap();

    let mut actual = map.iter().collect::<Vec<(i64, i64)>>();
    actual.sort();
    let expected: Vec<(i64, i64)> = vec![(1, 10), (2, 30), (4, 13), (5, 15)];

    assert_eq!(actual, expected);
}

#[test]
fn to_hash_map() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();
    map.insert(1, 10).unwrap();
    map.insert(5, 15).unwrap();
    map.insert(7, 20).unwrap();
    map.remove(&5).unwrap();

    let other = map.to_hash_map().unwrap();

    assert_eq!(other.len(), 2);
    assert_eq!(other.get(&1), Some(&10));
    assert_eq!(other.get(&5), None);
    assert_eq!(other.get(&7), Some(&20));
}

#[test]
fn to_hash_map_empty() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let map = StorageMap::<i64, i64>::try_from(storage).unwrap();
    let other = map.to_hash_map().unwrap();

    assert_eq!(other.len(), 0);
}

#[test]
fn try_from_storage_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let index;

    {
        let mut map = StorageMap::<i64, i64>::try_from(storage.clone()).unwrap();
        map.insert(1, 1).unwrap();
        map.insert(3, 2).unwrap();
        map.insert(5, 3).unwrap();
        map.remove(&3).unwrap();
        index = map.storage_index();
    }

    let map = StorageMap::<i64, i64>::try_from((storage, index)).unwrap();

    let mut expected = HashMap::<i64, i64>::new();
    expected.insert(1, 1);
    expected.insert(5, 3);

    assert_eq!(map.to_hash_map(), Ok(expected));
}

#[test]
fn try_from_storage_missing_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    assert_eq!(
        StorageMap::<i64, i64>::try_from((storage, StorageIndex::from(1_i64)))
            .err()
            .unwrap(),
        DbError::from("index '1' not found")
    );
}
