use agdb_db_error::DbError;
use agdb_storage::StorageFile;
use agdb_storage::StorageIndex;
use agdb_storage_vec::StorageVec;
use agdb_test_file::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn iter() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    assert_eq!(vec.iter().collect::<Vec<i64>>(), vec![1_i64, 3_i64, 5_i64]);
}

#[test]
fn is_empty() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

    assert!(vec.is_empty());

    vec.push(&1).unwrap();

    assert!(!vec.is_empty());
}

#[test]
fn len() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

    assert_eq!(vec.len(), 0);

    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    assert_eq!(vec.len(), 3)
}

#[test]
fn min_capacity() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

    assert_eq!(vec.capacity(), 0);

    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    assert_eq!(vec.capacity(), 64);
}

#[test]
fn to_vec() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    assert_eq!(vec.to_vec(), Ok(vec![1_i64, 3_i64, 5_i64]));
}

#[test]
fn try_from_storage_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let index;

    {
        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();
        index = vec.storage_index();
    }

    let vec = StorageVec::<i64>::try_from((storage, index)).unwrap();

    assert_eq!(vec.to_vec(), Ok(vec![1_i64, 3_i64, 5_i64]));
}

#[test]
fn try_from_storage_missing_index() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    assert_eq!(
        StorageVec::<i64>::try_from((storage, StorageIndex::from(1_i64)))
            .err()
            .unwrap(),
        DbError::from("index '1' not found")
    );
}
