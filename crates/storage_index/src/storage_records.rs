use crate::storage_record::StorageRecord;

pub struct StorageRecords {
    pub(crate) records: Vec<StorageRecord>,
}

impl StorageRecords {
    pub fn create(&mut self, position: u64, size: u64) -> i64 {
        let index;

        if let Some(free_index) = self.free_index() {
            index = free_index;
            self.records[free_index as usize] = StorageRecord {
                index: 0,
                position,
                size,
            };
        } else {
            index = self.records.len() as i64;
            self.records.push(StorageRecord {
                index: 0,
                position,
                size,
            });
        }

        index
    }

    pub fn get(&self, index: i64) -> Option<&StorageRecord> {
        if let Some(record) = self.records.get(index as usize) {
            if record.size != 0 {
                return Some(record);
            }
        }

        None
    }

    pub fn get_mut(&mut self, index: i64) -> Option<&mut StorageRecord> {
        if let Some(record) = self.records.get_mut(index as usize) {
            if record.size != 0 {
                return Some(record);
            }
        }

        None
    }

    pub fn indexes_by_position(&self) -> Vec<i64> {
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

    pub fn remove(&mut self, index: i64) {
        if let Some(_record) = self.get_mut(index) {
            self.add_free_index(index);
        }
    }

    pub(crate) fn add_free_index(&mut self, index: i64) {
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
