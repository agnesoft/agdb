use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use agdb_storage::FileStorage;
use agdb_storage::Storage;
use agdb_test_file::TestFile;

#[test]
fn resize_at_end_does_not_move() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

    let index = storage.insert(&1_i64).unwrap();
    let size = storage.size().unwrap();
    let value_size = storage.value_size(index).unwrap();

    storage.resize_value(index, value_size + 8).unwrap();

    assert_eq!(storage.size(), Ok(size + 8));
}

#[test]
fn resize_value_greater() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

    let index = storage.insert(&10_i64).unwrap();
    let expected_size = i64::serialized_size();

    assert_eq!(storage.value_size(index), Ok(expected_size));

    storage.resize_value(index, expected_size * 2).unwrap();

    assert_eq!(storage.value_size(index), Ok(expected_size * 2));
}

#[test]
fn resize_value_missing_index() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

    assert_eq!(
        storage.resize_value(1, 1),
        Err(DbError::from("index '1' not found"))
    );
}

#[test]
fn resize_value_same() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

    let index = storage.insert(&10_i64).unwrap();
    let expected_size = i64::serialized_size();

    assert_eq!(storage.value_size(index), Ok(expected_size));

    storage.resize_value(index, expected_size).unwrap();

    assert_eq!(storage.value_size(index), Ok(expected_size));
}

#[test]
fn resize_value_smaller() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

    let index = storage.insert(&10_i64).unwrap();
    let expected_size = i64::serialized_size();

    assert_eq!(storage.value_size(index), Ok(expected_size));

    storage.resize_value(index, expected_size / 2).unwrap();

    assert_eq!(storage.value_size(index), Ok(expected_size / 2));
}

#[test]
fn resize_value_zero() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

    let index = storage.insert(&10_i64).unwrap();
    let expected_size = i64::serialized_size();

    assert_eq!(storage.value_size(index), Ok(expected_size));

    assert_eq!(
        storage.resize_value(index, 0),
        Err(DbError::from("value size cannot be 0"))
    );
}

#[test]
fn resize_value_resizes_file() {
    let test_file = TestFile::new();

    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();
    let index = storage.insert(&3_i64).unwrap();
    let size = u64::serialized_size() + i64::serialized_size() * 3;
    storage.resize_value(index, size).unwrap();

    assert_eq!(storage.value::<Vec<i64>>(index), Ok(vec![0_i64; 3]));
}

#[test]
fn resize_value_invalidates_original_position() {
    let test_file = TestFile::new();

    let index;

    {
        let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();
        index = storage.insert(&10_i64).unwrap();
        storage.insert(&5_i64).unwrap();
        storage.resize_value(index, 1).unwrap();
        storage.remove(index).unwrap();
    }

    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

    assert_eq!(
        storage.value::<i64>(index),
        Err(DbError::from("index '1' not found"))
    );
}
