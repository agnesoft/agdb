use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage_index::StorageIndex;
use agdb_test_file::TestFile;

#[test]
fn value() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
    let index = storage.insert(&10_i64).unwrap();

    assert_eq!(storage.value::<i64>(&index), Ok(10_i64));
}

#[test]
fn value_at() {
    let test_file = TestFile::new();

    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
    let data = vec![1_i64, 2_i64, 3_i64];

    let index = storage.insert(&data).unwrap();
    let offset = u64::serialized_size() + i64::serialized_size();

    assert_eq!(storage.value_at::<i64>(&index, offset), Ok(2_i64));
}

#[test]
fn value_at_dynamic_size() {
    let test_file = TestFile::new();

    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
    let data = vec![2_i64, 1_i64, 2_i64];

    let index = storage.insert(&data).unwrap();
    let offset = u64::serialized_size();

    assert_eq!(
        storage.value_at::<Vec<i64>>(&index, offset),
        Ok(vec![1_i64, 2_i64])
    );
}

#[test]
fn value_at_of_missing_index() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    assert_eq!(
        storage.value_at::<i64>(&StorageIndex::from(1_i64), 8),
        Err(DbError::from("index '1' not found"))
    );
}

#[test]
fn value_at_out_of_bounds() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    let data = vec![1_i64, 2_i64];
    let index = storage.insert(&data).unwrap();
    let offset = (u64::serialized_size() + i64::serialized_size() * 2) as u64;

    assert_eq!(
        storage.value_at::<i64>(&index, offset),
        Err(DbError::from("deserialization error: value out of bounds"))
    );
}

#[test]
fn value_at_offset_overflow() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    let data = vec![1_i64, 2_i64];
    let index = storage.insert(&data).unwrap();
    let offset = (u64::serialized_size() + i64::serialized_size() * 3) as u64;

    assert_eq!(
        storage.value_at::<i64>(&index, offset),
        Err(DbError::from("deserialization error: offset out of bounds"))
    );
}

#[test]
fn value_of_missing_index() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    assert_eq!(
        storage.value::<i64>(&StorageIndex::from(1_i64)),
        Err(DbError::from("index '1' not found"))
    );
}

#[test]
fn value_out_of_bounds() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    let index = storage.insert(&10_i64).unwrap();

    assert_eq!(
        storage.value::<Vec<i64>>(&index),
        Err(DbError::from("i64 deserialization error: out of bounds"))
    );
}

#[test]
fn value_size() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    let index = storage.insert(&10_i64).unwrap();
    let expected_size = i64::serialized_size();

    assert_eq!(storage.value_size(&index), Ok(expected_size));
}

#[test]
fn value_size_of_missing_index() {
    let test_file = TestFile::new();
    let storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    assert_eq!(
        storage.value_size(&StorageIndex::from(1_i64)),
        Err(DbError::from("index '1' not found"))
    );
}
