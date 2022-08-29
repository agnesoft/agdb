use super::file_record::FileRecord;

#[allow(dead_code)]
pub(crate) struct FileRecords {
    records: Vec<FileRecord>,
    free_list: Vec<i64>,
}

#[allow(dead_code)]
impl FileRecords {
    pub(crate) fn create(&mut self, position: u64, size: u64) -> FileRecord {
        let record;

        if let Some(free_index) = self.free_list.pop() {
            let value = &mut self.records[free_index as usize];
            value.index = free_index;
            value.position = position;
            value.size = size;

            record = value.clone();
        } else {
            record = FileRecord {
                index: self.records.len() as i64,
                position,
                size,
            };

            self.records.push(record.clone());
        }

        record
    }

    pub(crate) fn get(&self, index: i64) -> Option<&FileRecord> {
        if let Some(record) = self.records.get(index as usize) {
            if 0 < record.index {
                return Some(record);
            }
        }

        None
    }

    pub(crate) fn get_mut(&mut self, index: i64) -> Option<&mut FileRecord> {
        if let Some(record) = self.records.get_mut(index as usize) {
            if 0 < record.index {
                return Some(record);
            }
        }

        None
    }

    pub(crate) fn remove(&mut self, index: i64) {
        if let Some(record) = self.get_mut(index) {
            record.index *= -1;
            self.free_list.push(index);
        }
    }
}

impl From<Vec<FileRecord>> for FileRecords {
    fn from(mut records: Vec<FileRecord>) -> Self {
        records.sort();

        let mut file_records;

        if let Some(last) = records.last() {
            file_records = FileRecords {
                records: vec![FileRecord::default(); last.index as usize + 1],
                ..Default::default()
            };
        } else {
            file_records = FileRecords::default();
        }

        let mut last_index = 0;

        for record in records {
            if record.index <= 0 {
                continue;
            }

            let index = record.index;

            for index in last_index + 1..record.index {
                file_records.free_list.push(index);
            }

            file_records.records[index as usize] = record;
            last_index = index;
        }

        file_records
    }
}

impl Default for FileRecords {
    fn default() -> Self {
        Self {
            records: vec![FileRecord::default()],
            free_list: Default::default(),
        }
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

        assert_eq!(file_records.get(record1.index), Some(&record1));
        assert_eq!(file_records.get(record2.index), Some(&record2));
        assert_eq!(file_records.get(record3.index), Some(&record3));
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
    fn get_mut_zero_index() {
        let mut file_records = FileRecords::default();

        assert_eq!(file_records.get_mut(0), None);
    }

    #[test]
    fn get_zero_index() {
        let file_records = FileRecords::default();

        assert_eq!(file_records.get(0), None);
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
