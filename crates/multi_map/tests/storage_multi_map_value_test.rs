use agdb_multi_map::StorageMultiMap;
use agdb_storage::StorageFile;
use agdb_test_file::TestFile;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn value_missing() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    assert_eq!(map.value(&0), Ok(None));
}

#[test]
fn values_at_end() {
    let test_file = TestFile::new();
    let storage = Rc::new(RefCell::new(
        StorageFile::try_from(test_file.file_name().clone()).unwrap(),
    ));
    let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

    map.insert(127, 10).unwrap();
    map.insert(255, 11).unwrap();
    map.insert(191, 12).unwrap();

    assert_eq!(map.value(&127), Ok(Some(10)));
    assert_eq!(map.value(&255), Ok(Some(11)));
    assert_eq!(map.value(&191), Ok(Some(12)));
}
