use std::collections::HashMap;

use super::file_record::FileRecord;

#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct FileRecords {
    records: HashMap<i64, FileRecord>,
    free_list: Vec<i64>,
}

#[allow(dead_code)]
impl FileRecords {
    pub(crate) fn get(&self, index: i64) -> Option<&FileRecord> {
        self.records.get(&index)
    }

    pub(crate) fn insert_new(&mut self, record: FileRecord) -> i64 {
        let mut index = self.records.len() as i64;

        if let Some(free_index) = self.free_list.pop() {
            index = free_index;
        }

        self.records.insert(index, record);
        index
    }

    pub(crate) fn insert(&mut self, index: i64, record: FileRecord) {
        self.records.insert(index, record);
    }

    pub(crate) fn remove(&mut self, index: i64) {
        self.records.remove(&index);
        self.free_list.push(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_records_can_be_default_constructed() {
        let _records = FileRecords::default();
    }

    #[test]
    fn insert_new() {
        let mut file_records = FileRecords::default();
        let record = FileRecord {
            pos: 32u64,
            size: 64u64,
        };

        let index = file_records.insert_new(record.clone());

        assert_eq!(file_records.get(index), Some(&record));
    }

    #[test]
    fn insert() {
        let mut file_records = FileRecords::default();
        let index = 2_i64;
        let record = FileRecord {
            pos: 8u64,
            size: 16u64,
        };

        file_records.insert(index, record.clone());

        assert_eq!(file_records.get(index), Some(&record));
    }

    #[test]
    fn remove() {
        let mut file_records = FileRecords::default();
        let index = file_records.insert_new(FileRecord::default());

        file_records.remove(index);

        assert_eq!(file_records.get(index), None);
    }

    #[test]
    fn reuse_indexes() {
        let mut file_records = FileRecords::default();
        let index = file_records.insert_new(FileRecord::default());
        file_records.remove(index);
        let index2 = file_records.insert_new(FileRecord::default());

        assert_eq!(index, index2);
    }
}
