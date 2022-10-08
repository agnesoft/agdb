use crate::storage_record::StorageRecord;
use crate::storage_records::StorageRecords;

impl From<Vec<StorageRecord>> for StorageRecords {
    fn from(mut records: Vec<StorageRecord>) -> Self {
        records.sort();

        let mut file_records;

        if let Some(last) = records.last() {
            file_records = StorageRecords {
                records: vec![StorageRecord::default(); last.index as usize + 1],
            };
        } else {
            file_records = StorageRecords::default();
        }

        let mut last_index = 0;

        for record in records {
            if record.index <= 0 {
                continue;
            }

            let index = record.index;

            for index in last_index + 1..record.index {
                file_records.add_free_index(index);
            }

            file_records.records[index as usize].position = record.position;
            file_records.records[index as usize].size = record.size;
            last_index = index;
        }

        file_records
    }
}
