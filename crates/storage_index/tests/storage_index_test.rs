use agdb_serialize::Serialize;
use agdb_storage_index::StorageIndex;

#[test]
fn is_valid() {
    assert!(!StorageIndex::default().is_valid());
    assert!(!StorageIndex::from(-1_i64).is_valid());

    assert!(StorageIndex::from(1_i64).is_valid());
    assert!(StorageIndex::from(100_i64).is_valid());
}

#[test]
fn ordering() {
    let mut indexes = vec![
        StorageIndex::default(),
        StorageIndex::from(100_i64),
        StorageIndex::from(-1_i64),
        StorageIndex::from(1_i64),
    ];

    indexes.sort();

    assert_eq!(
        indexes,
        vec![
            StorageIndex::from(-1_i64),
            StorageIndex::default(),
            StorageIndex::from(1_i64),
            StorageIndex::from(100_i64),
        ]
    )
}

#[test]
fn serialize() {
    let index = StorageIndex::from(1_i64);
    let bytes = index.serialize();
    let other = StorageIndex::deserialize(&bytes).unwrap();

    assert_eq!(index, other);
}
