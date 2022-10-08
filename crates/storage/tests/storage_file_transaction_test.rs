use agdb_db_error::DbError;
use agdb_storage::FileStorage;
use agdb_storage::Storage;
use agdb_test_file::TestFile;

#[test]
fn transaction_commit() {
    let test_file = TestFile::from("file_storage-transaction_commit.agdb");
    let index;

    {
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        storage.transaction();
        index = storage.insert(&1_i64).unwrap();
        storage.commit().unwrap();
        assert_eq!(storage.value::<i64>(index), Ok(1_i64));
    }

    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
    assert_eq!(storage.value::<i64>(index), Ok(1_i64));
}

#[test]
fn transaction_commit_no_transaction() {
    let test_file = TestFile::from("file_storage-transaction_commit_no_transaction.agdb");
    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
    assert_eq!(storage.commit(), Ok(()));
}

#[test]
fn transaction_unfinished() {
    let test_file = TestFile::new();
    let index;

    {
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        storage.transaction();
        index = storage.insert(&1_i64).unwrap();
        assert_eq!(storage.value::<i64>(index), Ok(1_i64));
    }

    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
    assert_eq!(
        storage.value::<i64>(index),
        Err(DbError::from(format!("index '{}' not found", index)))
    );
}

#[test]
fn transaction_nested_unfinished() {
    let test_file = TestFile::new();
    let index;

    {
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        storage.transaction();
        storage.transaction();
        index = storage.insert(&1_i64).unwrap();
        assert_eq!(storage.value::<i64>(index), Ok(1_i64));
        storage.commit().unwrap();
    }

    let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
    assert_eq!(
        storage.value::<i64>(index),
        Err(DbError::from(format!("index '{}' not found", index)))
    );
}
