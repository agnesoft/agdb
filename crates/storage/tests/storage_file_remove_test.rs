use agdb_db_error::DbError;
use agdb_storage::FileStorage;
use agdb_storage::Storage;
use agdb_test_file::TestFile;

#[test]
fn remove() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&1_i64).unwrap();
    storage.remove(index).unwrap();

    assert_eq!(
        storage.value::<i64>(index),
        Err(DbError::from("index '1' not found"))
    );
}

#[test]
fn remove_missing_index() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

    assert_eq!(
        storage.remove(1_i64),
        Err(DbError::from("index '1' not found"))
    );
}
