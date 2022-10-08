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
