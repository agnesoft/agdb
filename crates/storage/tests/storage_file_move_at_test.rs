use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage_index::StorageIndex;
use agdb_test_utilities::TestFile;

#[test]
fn move_at() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
    let offset_from = (u64::serialized_size() + i64::serialized_size() * 2) as u64;
    let offset_to = (u64::serialized_size() + i64::serialized_size()) as u64;
    let size = u64::serialized_size();

    storage
        .move_at(&index, offset_from, offset_to, size)
        .unwrap();

    assert_eq!(
        storage.value::<Vec<i64>>(&index).unwrap(),
        vec![1_i64, 3_i64, 0_i64]
    )
}

#[test]
fn move_at_beyond_end() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
    let offset_from = (u64::serialized_size() + i64::serialized_size()) as u64;
    let offset_to = (u64::serialized_size() + i64::serialized_size() * 4) as u64;
    let size = u64::serialized_size();

    storage
        .move_at(&index, offset_from, offset_to, size)
        .unwrap();

    storage.insert_at(&index, 0, &5_u64).unwrap();

    assert_eq!(
        storage.value::<Vec<i64>>(&index).unwrap(),
        vec![1_i64, 0_i64, 3_i64, 0_i64, 2_i64]
    )
}

#[test]
fn move_at_missing_index() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

    assert_eq!(
        storage.move_at(&StorageIndex::from(1_i64), 0, 1, 10),
        Err(DbError::from("index '1' not found"))
    );
}

#[test]
fn move_at_same_offset() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();

    assert_eq!(storage.move_at(&index, 0, 0, 10), Ok(()));
    assert_eq!(
        storage.value::<Vec<i64>>(&index).unwrap(),
        vec![1_i64, 2_i64, 3_i64]
    );
}

#[test]
fn move_at_size_out_of_bounds() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
    let offset_from = (u64::serialized_size() + i64::serialized_size() * 3) as u64;
    let offset_to = (u64::serialized_size() + i64::serialized_size() * 2) as u64;
    let size = (u64::serialized_size() * 10) as u64;

    assert_eq!(
        storage.move_at(&index, offset_from, offset_to, size),
        Err(DbError::from("move size out of bounds"))
    );
}

#[test]
fn move_at_zero_size() {
    let test_file = TestFile::new();
    let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();

    assert_eq!(storage.move_at(&index, 0, 1, 0), Ok(()));
    assert_eq!(
        storage.value::<Vec<i64>>(&index).unwrap(),
        vec![1_i64, 2_i64, 3_i64]
    );
}
