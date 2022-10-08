use agdb_serialize::Serialize;
use agdb_storage_index::StorageIndex;
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

#[test]
fn serialize() {
    let bytes = StorageRecord {
        index: StorageIndex::from(1_i64),
        position: 64,
        size: 128,
    }
    .serialize();
    let record = StorageRecord::deserialize(&bytes).unwrap();

    assert_eq!(record.index, StorageIndex::from(1_i64));
    assert_eq!(record.position, 0);
    assert_eq!(record.size, 128);
}
