use crate::storage_index::StorageIndex;
use crate::storage_record::StorageRecord;
use crate::storage_records::StorageRecords;

impl From<Vec<StorageRecord>> for StorageRecords {
    fn from(mut records: Vec<StorageRecord>) -> Self {
        records.sort();

        let mut file_records = StorageRecords::default();

        if let Some(last) = records.last() {
            if last.index.is_valid() {
                file_records = StorageRecords {
                    records: vec![StorageRecord::default(); last.index.as_usize() + 1],
                };
            }
        }

        let mut last_index = 0;

        for record in records {
            if !record.index.is_valid() {
                continue;
            }

            for index in (last_index + 1)..record.index.value() {
                file_records.add_free_index(&StorageIndex::from(index));
            }

            file_records.records[record.index.as_usize()].position = record.position;
            file_records.records[record.index.as_usize()].size = record.size;
            last_index = record.index.value();
        }

        file_records
    }
}
