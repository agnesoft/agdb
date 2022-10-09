use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage_vec::StorageVec;
use agdb_test_file::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn resize_larger() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    vec.resize(6).unwrap();

    assert_eq!(
        storage
            .borrow_mut()
            .value::<Vec::<i64>>(&vec.storage_index()),
        Ok(vec![1_i64, 3_i64, 5_i64, 0, 0, 0])
    );
}

#[test]
fn resize_over_capacity() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    vec.resize(100).unwrap();

    let mut expected = vec![0_i64; 100];
    expected[0] = 1;
    expected[1] = 3;
    expected[2] = 5;

    assert_eq!(vec.len(), 100);
    assert_eq!(vec.capacity(), 100);

    assert_eq!(
        storage
            .borrow_mut()
            .value::<Vec::<i64>>(&vec.storage_index()),
        Ok(expected)
    );
}

#[test]
fn resize_same() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    vec.resize(3).unwrap();

    assert_eq!(
        storage
            .borrow_mut()
            .value::<Vec::<i64>>(&vec.storage_index()),
        Ok(vec![1_i64, 3_i64, 5_i64])
    );
}

#[test]
fn resize_smaller() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    vec.resize(1).unwrap();

    assert_eq!(
        storage
            .borrow_mut()
            .value::<Vec::<i64>>(&vec.storage_index()),
        Ok(vec![1_i64])
    );
}
