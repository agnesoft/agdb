use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use agdb_storage::FileStorage;
use agdb_storage::Storage;
use agdb_test_file::TestFile;

#[test]
fn insert() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&10_i64);

    assert_eq!(index, Ok(1));
}

#[test]
fn insert_at() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
    let offset = (u64::serialized_size() + i64::serialized_size()) as u64;
    storage.insert_at(index, offset, &10_i64).unwrap();

    assert_eq!(
        storage.value::<Vec<i64>>(index).unwrap(),
        vec![1_i64, 10_i64, 3_i64]
    );
}

#[test]
fn insert_at_missing_index() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

    assert_eq!(
        storage.insert_at(1, 8, &1_i64),
        Err(DbError::from("index '1' not found"))
    );
}

#[test]
fn insert_at_value_end() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
    let offset = (u64::serialized_size() + i64::serialized_size() * 3) as u64;
    storage.insert_at(index, 0, &4_u64).unwrap();
    storage.insert_at(index, offset, &10_i64).unwrap();

    assert_eq!(
        storage.value::<Vec<i64>>(index).unwrap(),
        vec![1_i64, 2_i64, 3_i64, 10_i64]
    );
}

#[test]
fn insert_at_beyond_end() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
    let offset = (u64::serialized_size() + i64::serialized_size() * 4) as u64;
    storage.insert_at(index, 0, &5_u64).unwrap();
    storage.insert_at(index, offset, &10_i64).unwrap();

    assert_eq!(
        storage.value::<Vec<i64>>(index).unwrap(),
        vec![1_i64, 2_i64, 3_i64, 0_i64, 10_i64]
    );
}

#[test]
fn insert_at_bytes() {
    let test_file = TestFile::new();
    let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

    let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
    let offset = (u64::serialized_size() + i64::serialized_size()) as u64;
    let size = i64::serialized_size() * 2;

    storage
        .insert_at(index, offset, &vec![0_u8; size as usize])
        .unwrap();

    assert_eq!(
        storage.value::<Vec<i64>>(index).unwrap(),
        vec![1_i64, 0_i64, 0_i64]
    );
}
