use agdb_db_error::DbError;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage_vec::StorageVec;
use agdb_test_file::TestFile;

#[test]
fn remove() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    vec.remove(1).unwrap();

    assert_eq!(vec.to_vec(), Ok(vec![1, 5]));
}

#[test]
fn remove_at_end() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    vec.remove(2).unwrap();

    assert_eq!(vec.to_vec(), Ok(vec![1, 3]));
}

#[test]
fn remove_index_out_of_bounds() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

    assert_eq!(vec.remove(0), Err(DbError::from("index out of bounds")));
}

#[test]
fn remove_size_updated() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    vec.remove(1).unwrap();

    assert_eq!(
        storage
            .borrow_mut()
            .value::<Vec::<i64>>(&vec.storage_index()),
        Ok(vec![1_i64, 5_i64])
    );
}
