use crate::storage_record::StorageRecord;
use crate::storage_records::StorageRecords;

impl Default for StorageRecords {
    fn default() -> Self {
        Self {
            records: vec![StorageRecord::default()],
        }
    }
}
