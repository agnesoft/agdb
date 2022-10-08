use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use agdb_storage::FileStorage;
use agdb_storage::Storage;
use agdb_test_file::TestFile;

#[test]
fn shrink_to_fit() {
    let test_file = TestFile::new();

    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
    let index1 = storage.insert(&1_i64).unwrap();
    let index2 = storage.insert(&2_i64).unwrap();
    let index3 = storage.insert(&3_i64).unwrap();
    storage.remove(index2).unwrap();
    storage.shrink_to_fit().unwrap();

    let actual_size = std::fs::metadata(test_file.file_name()).unwrap().len();
    let expected_size = (u64::serialized_size() * 2) * 2 + i64::serialized_size() * 2;

    assert_eq!(actual_size, expected_size);
    assert_eq!(storage.value(index1), Ok(1_i64));
    assert_eq!(storage.value(index3), Ok(3_i64));
}

#[test]
fn shrink_to_fit_no_change() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
    let index1 = storage.insert(&1_i64).unwrap();
    let index2 = storage.insert(&2_i64).unwrap();
    let index3 = storage.insert(&3_i64).unwrap();

    let actual_size = std::fs::metadata(test_file.file_name()).unwrap().len();

    storage.shrink_to_fit().unwrap();

    assert_eq!(
        actual_size,
        std::fs::metadata(test_file.file_name()).unwrap().len()
    );
    assert_eq!(storage.value(index1), Ok(1_i64));
    assert_eq!(storage.value(index2), Ok(2_i64));
    assert_eq!(storage.value(index3), Ok(3_i64));
}

#[test]
fn shrink_to_fit_uncommitted() {
    let test_file = TestFile::new();

    let expected_size;
    let index1;
    let index2;
    let index3;

    {
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        index1 = storage.insert(&1_i64).unwrap();
        index2 = storage.insert(&2_i64).unwrap();
        index3 = storage.insert(&3_i64).unwrap();
        storage.remove(index2).unwrap();

        expected_size = std::fs::metadata(test_file.file_name()).unwrap().len();

        storage.transaction();
        storage.shrink_to_fit().unwrap();
    }

    let actual_size = std::fs::metadata(test_file.file_name()).unwrap().len();
    assert_eq!(actual_size, expected_size);

    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
    assert_eq!(storage.value(index1), Ok(1_i64));
    assert_eq!(
        storage.value::<i64>(index2),
        Err(DbError::from(format!("index '{}' not found", index2)))
    );
    assert_eq!(storage.value(index3), Ok(3_i64));
}
