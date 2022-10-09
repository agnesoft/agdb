use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage_vec::StorageVec;
use agdb_test_utilities::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn push() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));

    let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
    vec.push(&1).unwrap();
    vec.push(&3).unwrap();
    vec.push(&5).unwrap();

    assert_eq!(
        storage
            .borrow_mut()
            .value::<Vec::<i64>>(&vec.storage_index()),
        Ok(vec![1_i64, 3_i64, 5_i64])
    );
}
