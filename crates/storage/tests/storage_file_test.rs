use agdb_db_error::DbError;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage_index::StorageIndex;
use agdb_test_utilities::TestFile;

#[test]
fn bad_file() {
    assert!(StorageFile::try_from("/a/").is_err());
}

#[test]
fn restore_from_file() {
    let test_file = TestFile::new();
    let value1 = vec![1_i64, 2_i64, 3_i64];
    let value2 = 64_u64;
    let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
    let index1;
    let index2;
    let index3;

    {
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        index1 = storage.insert(&value1).unwrap();
        index2 = storage.insert(&value2).unwrap();
        index3 = storage.insert(&value3).unwrap();
    }

    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    assert_eq!(storage.value::<Vec<i64>>(&index1), Ok(value1));
    assert_eq!(storage.value::<u64>(&index2), Ok(value2));
    assert_eq!(storage.value::<Vec<i64>>(&index3), Ok(value3));
}

#[test]
fn restore_from_file_with_removed_index() {
    let test_file = TestFile::new();
    let value1 = vec![1_i64, 2_i64, 3_i64];
    let value2 = 64_u64;
    let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
    let index1;
    let index2;
    let index3;

    {
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        index1 = storage.insert(&value1).unwrap();
        index2 = storage.insert(&value2).unwrap();
        index3 = storage.insert(&value3).unwrap();
        storage.remove(&index2).unwrap();
    }

    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    assert_eq!(storage.value::<Vec<i64>>(&index1), Ok(value1));
    assert_eq!(
        storage.value::<u64>(&StorageIndex::default()),
        Err(DbError::from(format!("index '{}' not found", 0)))
    );
    assert_eq!(
        storage.value::<u64>(&index2),
        Err(DbError::from(format!(
            "index '{}' not found",
            index2.value()
        )))
    );
    assert_eq!(storage.value::<Vec<i64>>(&index3), Ok(value3));
}

#[test]
fn restore_from_file_with_all_indexes_removed() {
    let test_file = TestFile::new();
    let value1 = vec![1_i64, 2_i64, 3_i64];
    let value2 = 64_u64;
    let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
    let index1;
    let index2;
    let index3;

    {
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        index1 = storage.insert(&value1).unwrap();
        index2 = storage.insert(&value2).unwrap();
        index3 = storage.insert(&value3).unwrap();
        storage.remove(&index1).unwrap();
        storage.remove(&index2).unwrap();
        storage.remove(&index3).unwrap();
    }

    let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

    assert_eq!(
        storage.value::<u64>(&StorageIndex::default()),
        Err(DbError::from(format!("index '{}' not found", 0)))
    );
    assert_eq!(
        storage.value::<Vec<i64>>(&index1),
        Err(DbError::from(format!(
            "index '{}' not found",
            index1.value()
        )))
    );
    assert_eq!(
        storage.value::<u64>(&index2),
        Err(DbError::from(format!(
            "index '{}' not found",
            index2.value()
        )))
    );
    assert_eq!(
        storage.value::<Vec<i64>>(&index3),
        Err(DbError::from(format!(
            "index '{}' not found",
            index3.value()
        )))
    );
}
