use agdb_storage_index::StorageRecord;
use std::cmp::Ordering;

#[test]
fn derived_from_debug() {
    let record = StorageRecord::default();
    format!("{:?}", record);
}

#[test]
fn derived_from_ord() {
    let record = StorageRecord::default();
    assert_eq!(record.cmp(&record), Ordering::Equal);
}
