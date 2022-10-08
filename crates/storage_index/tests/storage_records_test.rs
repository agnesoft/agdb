use agdb_storage_index::StorageRecord;
use agdb_storage_index::StorageRecords;

#[test]
fn create() {
    let mut file_records = StorageRecords::default();

    let index = file_records.create(0, 0);

    assert_eq!(index, 1_i64);
}

#[test]
fn default_constructed() {
    let _records = StorageRecords::default();
}

#[test]
fn from_records() {
    let index1 = 2;
    let index2 = 1;
    let index3 = 3;

    let file_records = StorageRecords::from(vec![
        StorageRecord {
            index: index1,
            position: 8,
            size: 16,
        },
        StorageRecord {
            index: index2,
            position: 24,
            size: 16,
        },
        StorageRecord {
            index: index3,
            position: 40,
            size: 16,
        },
    ]);

    assert_eq!(
        file_records.get(index1),
        Some(&StorageRecord {
            index: 0,
            position: 8,
            size: 16
        })
    );
    assert_eq!(
        file_records.get(index2),
        Some(&StorageRecord {
            index: 0,
            position: 24,
            size: 16
        })
    );
    assert_eq!(
        file_records.get(index3),
        Some(&StorageRecord {
            index: 0,
            position: 40,
            size: 16
        })
    );
}

#[test]
fn from_records_with_index_gaps() {
    let record1 = StorageRecord {
        index: 5,
        position: 24,
        size: 16,
    };
    let record2 = StorageRecord {
        index: 1,
        position: 40,
        size: 16,
    };
    let record3 = StorageRecord {
        index: 2,
        position: 40,
        size: 16,
    };

    let mut file_records = StorageRecords::from(vec![record1, record2, record3]);

    let index1 = file_records.create(2, 2);
    let index2 = file_records.create(4, 4);
    let index3 = file_records.create(6, 6);

    assert_eq!(index1, 4);
    assert_eq!(index2, 3);
    assert_eq!(index3, 6);
}

#[test]
fn from_records_with_removed_index() {
    let record1 = StorageRecord {
        index: 1,
        position: 24,
        size: 16,
    };
    let record2 = StorageRecord {
        index: -2,
        position: 40,
        size: 16,
    };
    let record3 = StorageRecord {
        index: 3,
        position: 40,
        size: 16,
    };

    let file_records = StorageRecords::from(vec![record1, record2, record3]);

    assert_eq!(file_records.get(0), None);
}

#[test]
fn get() {
    let mut file_records = StorageRecords::default();
    let position = 32_u64;
    let size = 64_u64;

    let index = file_records.create(position, size);
    let expected_record = StorageRecord {
        index: 0,
        position,
        size,
    };

    assert_eq!(file_records.get(index), Some(&expected_record));
}

#[test]
fn get_mut() {
    let mut file_records = StorageRecords::default();
    let position = 32_u64;
    let size = 64_u64;

    let index = file_records.create(position, size);
    let mut expected_record = StorageRecord {
        index: 0,
        position,
        size,
    };

    assert_eq!(file_records.get_mut(index), Some(&mut expected_record));
}

#[test]
fn get_mut_invalid_index() {
    let mut file_records = StorageRecords::default();

    assert_eq!(file_records.get_mut(-1), None);
}

#[test]
fn get_mut_zero_index() {
    let mut file_records = StorageRecords::default();

    assert_eq!(file_records.get_mut(0), None);
}

#[test]
fn get_zero_index() {
    let file_records = StorageRecords::default();

    assert_eq!(file_records.get(0), None);
}

#[test]
fn indexes_by_position() {
    let mut file_records = StorageRecords::default();
    let index1 = file_records.create(30, 8);
    let index2 = file_records.create(20, 8);
    let index3 = file_records.create(10, 8);
    file_records.remove(index2);

    assert_eq!(file_records.indexes_by_position(), vec![index3, index1]);
}

#[test]
fn remove() {
    let mut file_records = StorageRecords::default();
    let index = file_records.create(8u64, 16u64);

    file_records.remove(index);

    assert_eq!(file_records.get(index), None);
}

#[test]
fn remove_invalid_index() {
    let mut file_records = StorageRecords::default();
    let record = StorageRecord {
        index: 0,
        position: 8u64,
        size: 16u64,
    };
    let index = file_records.create(record.position, record.size);

    file_records.remove(-1);

    assert_eq!(file_records.get(index), Some(&record));
}

#[test]
fn reuse_indexes() {
    let mut file_records = StorageRecords::default();
    let index = file_records.create(8u64, 16u64);
    file_records.remove(index);
    let index2 = file_records.create(16u64, 32u64);

    assert_eq!(index, index2);
}
