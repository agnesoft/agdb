use agdb_db_error::DbError;
use agdb_storage::StorageFile;
use agdb_storage_vec::StorageVec;
use agdb_test_file::TestFile;

#[test]
fn set_value() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    vec.set_value(1, &10).unwrap();

    assert_eq!(vec.value(0), Ok(1));
    assert_eq!(vec.value(1), Ok(10));
    assert_eq!(vec.value(2), Ok(5));
}

#[test]
fn set_value_out_of_bounds() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

    assert_eq!(
        vec.set_value(0, &10),
        Err(DbError::from("index out of bounds"))
    );
}
