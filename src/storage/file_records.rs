use super::file_record::FileRecord;
use super::file_record_full::FileRecordFull;

#[allow(dead_code)]
pub(crate) struct FileRecords {
    records: Vec<FileRecord>,
}

#[allow(dead_code)]
impl FileRecords {
    pub(crate) fn create(&mut self, position: u64, size: u64) -> i64 {
        let index;

        if let Some(free_index) = self.free_index() {
            index = free_index;
            self.records[free_index as usize] = FileRecord { position, size };
        } else {
            index = self.records.len() as i64;
            self.records.push(FileRecord { position, size });
        }

        index
    }

    pub(crate) fn get(&self, index: i64) -> Option<&FileRecord> {
        if let Some(record) = self.records.get(index as usize) {
            if record.size != 0 {
                return Some(record);
            }
        }

        None
    }

    pub(crate) fn get_mut(&mut self, index: i64) -> Option<&mut FileRecord> {
        if let Some(record) = self.records.get_mut(index as usize) {
            if record.size != 0 {
                return Some(record);
            }
        }

        None
    }

    pub(crate) fn indexes_by_position(&self) -> Vec<i64> {
        let mut indexes = Vec::<i64>::new();

        for index in 1..self.records.len() {
            if self.records[index].size != 0 {
                indexes.push(index as i64);
            }
        }

        indexes.sort_by(|left, right| {
            self.records[*left as usize]
                .position
                .cmp(&self.records[*right as usize].position)
        });

        indexes
    }

    pub(crate) fn remove(&mut self, index: i64) {
        if let Some(_record) = self.get_mut(index) {
            self.add_free_index(index);
        }
    }

    fn add_free_index(&mut self, index: i64) {
        self.records[index as usize].position = self.records[0].position;
        self.records[index as usize].size = 0;
        self.records[0].position = index as u64;
    }

    fn free_index(&mut self) -> Option<i64> {
        let free = self.records[0].position;

        if free != 0 {
            self.records[0].position = self.records[free as usize].position;
            return Some(free as i64);
        }

        None
    }
}

impl From<Vec<FileRecordFull>> for FileRecords {
    fn from(mut records: Vec<FileRecordFull>) -> Self {
        records.sort();

        let mut file_records;

        if let Some(last) = records.last() {
            file_records = FileRecords {
                records: vec![FileRecord::default(); last.index as usize + 1],
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
                file_records.add_free_index(index);
            }

            file_records.records[index as usize].position = record.position;
            file_records.records[index as usize].size = record.size;
            last_index = index;
        }

        file_records
    }
}

impl Default for FileRecords {
    fn default() -> Self {
        Self {
            records: vec![FileRecord::default()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let mut file_records = FileRecords::default();

        let index = file_records.create(0, 0);

        assert_eq!(index, 1_i64);
    }

    #[test]
    fn default_constructed() {
        let _records = FileRecords::default();
    }

    #[test]
    fn from_records() {
        let index1 = 2;
        let index2 = 1;
        let index3 = 3;

        let file_records = FileRecords::from(vec![
            FileRecordFull {
                index: index1,
                position: 8,
                size: 16,
            },
            FileRecordFull {
                index: index2,
                position: 24,
                size: 16,
            },
            FileRecordFull {
                index: index3,
                position: 40,
                size: 16,
            },
        ]);

        assert_eq!(
            file_records.get(index1),
            Some(&FileRecord {
                position: 8,
                size: 16
            })
        );
        assert_eq!(
            file_records.get(index2),
            Some(&FileRecord {
                position: 24,
                size: 16
            })
        );
        assert_eq!(
            file_records.get(index3),
            Some(&FileRecord {
                position: 40,
                size: 16
            })
        );
    }

    #[test]
    fn from_records_with_index_gaps() {
        let record1 = FileRecordFull {
            index: 5,
            position: 24,
            size: 16,
        };
        let record2 = FileRecordFull {
            index: 1,
            position: 40,
            size: 16,
        };
        let record3 = FileRecordFull {
            index: 2,
            position: 40,
            size: 16,
        };

        let mut file_records = FileRecords::from(vec![record1, record2, record3]);

        let index1 = file_records.create(2, 2);
        let index2 = file_records.create(4, 4);
        let index3 = file_records.create(6, 6);

        assert_eq!(index1, 4);
        assert_eq!(index2, 3);
        assert_eq!(index3, 6);
    }

    #[test]
    fn from_records_with_removed_index() {
        let record1 = FileRecordFull {
            index: 1,
            position: 24,
            size: 16,
        };
        let record2 = FileRecordFull {
            index: -2,
            position: 40,
            size: 16,
        };
        let record3 = FileRecordFull {
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

        let index = file_records.create(position, size);
        let expected_record = FileRecord { position, size };

        assert_eq!(file_records.get(index), Some(&expected_record));
    }

    #[test]
    fn get_mut() {
        let mut file_records = FileRecords::default();
        let position = 32_u64;
        let size = 64_u64;

        let index = file_records.create(position, size);
        let mut expected_record = FileRecord { position, size };

        assert_eq!(file_records.get_mut(index), Some(&mut expected_record));
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
    fn indexes_by_position() {
        let mut file_records = FileRecords::default();
        let index1 = file_records.create(30, 8);
        let index2 = file_records.create(20, 8);
        let index3 = file_records.create(10, 8);
        file_records.remove(index2);

        assert_eq!(file_records.indexes_by_position(), vec![index3, index1]);
    }

    #[test]
    fn remove() {
        let mut file_records = FileRecords::default();
        let index = file_records.create(8u64, 16u64);

        file_records.remove(index);

        assert_eq!(file_records.get(index), None);
    }

    #[test]
    fn reuse_indexes() {
        let mut file_records = FileRecords::default();
        let index = file_records.create(8u64, 16u64);
        file_records.remove(index);
        let index2 = file_records.create(16u64, 32u64);

        assert_eq!(index, index2);
    }
}
