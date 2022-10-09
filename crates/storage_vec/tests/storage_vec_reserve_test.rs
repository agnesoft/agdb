use agdb_storage::StorageFile;
use agdb_storage_vec::StorageVec;
use agdb_test_file::TestFile;

#[test]
fn reserve_larger() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
    assert_eq!(vec.capacity(), 0);

    vec.reserve(20).unwrap();

    assert_eq!(vec.capacity(), 20);
}

#[test]
fn reserve_smaller() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
    vec.reserve(20).unwrap();
    vec.reserve(10).unwrap();

    assert_eq!(vec.capacity(), 20);
}
