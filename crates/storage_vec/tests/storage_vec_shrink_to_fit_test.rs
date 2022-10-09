use agdb_storage::StorageFile;
use agdb_storage_vec::StorageVec;
use agdb_test_file::TestFile;

#[test]
fn shrink_to_fit() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    assert_eq!(vec.capacity(), 64);

    vec.shrink_to_fit().unwrap();

    assert_eq!(vec.capacity(), 3);

    vec.shrink_to_fit().unwrap();

    assert_eq!(vec.capacity(), 3);
}

#[test]
fn shrink_to_fit_empty() {
    let test_file = TestFile::new();
    let storage = std::rc::Rc::new(std::cell::RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

    assert_eq!(vec.capacity(), 0);

    vec.shrink_to_fit().unwrap();

    assert_eq!(vec.capacity(), 0);
}
