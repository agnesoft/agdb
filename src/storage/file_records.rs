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
    pub(crate) fn create(&mut self, position: u64, size: u64) -> FileRecord {
        let index;

        if let Some(free_index) = self.free_list.pop() {
            index = free_index;
        } else {
            index = self.records.len() as i64 + 1;
        }

        let record = FileRecord {
            index,
            position,
            size,
        };

        self.records.insert(index, record.clone());
        record
    }

    pub(crate) fn get(&self, index: i64) -> Option<&FileRecord> {
        self.records.get(&index)
    }

    pub(crate) fn get_mut(&mut self, index: i64) -> Option<&mut FileRecord> {
        self.records.get_mut(&index)
    }

    pub(crate) fn remove(&mut self, index: i64) {
        self.records.remove(&index);
        self.free_list.push(index);
    }
}

impl From<Vec<FileRecord>> for FileRecords {
    fn from(mut records: Vec<FileRecord>) -> Self {
        let mut last_index = 0;
        let mut file_records = FileRecords::default();

        records.sort();

        for record in records {
            if record.index < 0 {
                continue;
            }

            for index in (last_index + 1)..record.index {
                file_records.free_list.push(index);
            }

            last_index = record.index;
            file_records.records.insert(record.index, record);
        }

        file_records
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let mut file_records = FileRecords::default();
        let position = 32_u64;
        let size = 64_u64;

        let actual_record = file_records.create(position, size);
        let expected_record = FileRecord {
            index: 1_i64,
            position,
            size,
        };

        assert_eq!(actual_record, expected_record);
    }

    #[test]
    fn default_constructed() {
        let _records = FileRecords::default();
    }

    #[test]
    fn from_records() {
        let record1 = FileRecord {
            index: 2,
            position: 8,
            size: 16,
        };
        let record2 = FileRecord {
            index: 1,
            position: 24,
            size: 16,
        };
        let record3 = FileRecord {
            index: 3,
            position: 40,
            size: 16,
        };

        let file_records =
            FileRecords::from(vec![record1.clone(), record2.clone(), record3.clone()]);

        assert_eq!(file_records.get(2), Some(&record1));
        assert_eq!(file_records.get(1), Some(&record2));
        assert_eq!(file_records.get(3), Some(&record3));
    }

    #[test]
    fn from_records_with_index_gaps() {
        let record1 = FileRecord {
            index: 5,
            position: 24,
            size: 16,
        };
        let record2 = FileRecord {
            index: 1,
            position: 40,
            size: 16,
        };
        let record3 = FileRecord {
            index: 2,
            position: 40,
            size: 16,
        };

        let mut file_records = FileRecords::from(vec![record1, record2, record3]);

        let new_record1 = file_records.create(2, 2);
        let new_record2 = file_records.create(4, 4);
        let new_record3 = file_records.create(6, 6);

        assert_eq!(new_record1.index, 4);
        assert_eq!(new_record2.index, 3);
        assert_eq!(new_record3.index, 6);
    }

    #[test]
    fn from_records_with_removed_index() {
        let record1 = FileRecord {
            index: 1,
            position: 24,
            size: 16,
        };
        let record2 = FileRecord {
            index: -2,
            position: 40,
            size: 16,
        };
        let record3 = FileRecord {
            index: 3,
            position: 40,
            size: 16,
        };

        let file_records = FileRecords::from(vec![record1, record2, record3]);

        assert_eq!(file_records.get(0), None);
    }

    #[test]
    fn get() {
        let mut file_records = FileRecords::default();
        let position = 32_u64;
        let size = 64_u64;

        file_records.create(position, size);
        let expected_record = FileRecord {
            index: 1,
            position,
            size,
        };

        assert_eq!(
            file_records.get(expected_record.index),
            Some(&expected_record)
        );
    }

    #[test]
    fn get_mut() {
        let mut file_records = FileRecords::default();
        let position = 32_u64;
        let size = 64_u64;

        file_records.create(position, size);
        let mut expected_record = FileRecord {
            index: 1,
            position,
            size,
        };

        assert_eq!(
            file_records.get_mut(expected_record.index),
            Some(&mut expected_record)
        );
    }

    #[test]
    fn remove() {
        let mut file_records = FileRecords::default();
        let record = file_records.create(8u64, 16u64);

        file_records.remove(record.index);

        assert_eq!(file_records.get(record.index), None);
    }

    #[test]
    fn reuse_indexes() {
        let mut file_records = FileRecords::default();
        let record = file_records.create(8u64, 16u64);
        file_records.remove(record.index);
        let record2 = file_records.create(16u64, 32u64);

        assert_eq!(record.index, record2.index);
    }
}
