use agdb_db_error::DbError;
use agdb_storage::StorageFile;
use agdb_storage_vec::StorageVec;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn value() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    assert_eq!(vec.value(0), Ok(1));
    assert_eq!(vec.value(1), Ok(3));
    assert_eq!(vec.value(2), Ok(5));
}

#[test]
fn value_out_of_bounds() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let vec = StorageVec::<i64>::try_from(storage).unwrap();

    assert_eq!(vec.value(0), Err(DbError::from("index out of bounds")));
}
