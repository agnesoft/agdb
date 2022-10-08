use crate::storage_index::StorageIndex;
use crate::storage_record::StorageRecord;

pub struct StorageRecords {
    pub(crate) records: Vec<StorageRecord>,
}

impl StorageRecords {
    pub fn create(&mut self, position: u64, size: u64) -> StorageRecord {
        let index;

        if let Some(free_index) = self.free_index() {
            index = free_index.as_usize();
            self.records[index] = StorageRecord {
                index: free_index,
                position,
                size,
            };
        } else {
            index = self.records.len();
            self.records.push(StorageRecord {
                index: StorageIndex::from(index),
                position,
                size,
            });
        }

        self.records[index].clone()
    }

    pub fn get(&self, index: &StorageIndex) -> Option<&StorageRecord> {
        if let Some(record) = self.records.get(index.as_usize()) {
            if record.size != 0 {
                return Some(record);
            }
        }

        None
    }

    pub fn get_mut(&mut self, index: &StorageIndex) -> Option<&mut StorageRecord> {
        if let Some(record) = self.records.get_mut(index.as_usize()) {
            if record.size != 0 {
                return Some(record);
            }
        }

        None
    }

    pub fn indexes_by_position(&self) -> Vec<StorageIndex> {
        let mut indexes = Vec::<StorageIndex>::new();

        for index in 1..self.records.len() {
            if self.records[index].size != 0 {
                indexes.push(StorageIndex::from(index));
            }
        }

        indexes.sort_by(|left, right| {
            self.records[left.as_usize()]
                .position
                .cmp(&self.records[right.as_usize()].position)
        });

        indexes
    }

    pub fn remove(&mut self, index: &StorageIndex) {
        if let Some(_record) = self.get_mut(index) {
            self.add_free_index(index);
        }
    }

    pub(crate) fn add_free_index(&mut self, index: &StorageIndex) {
        self.records[index.as_usize()].position = self.records[0].position;
        self.records[index.as_usize()].size = 0;
        self.records[0].position = index.as_u64();
    }

    fn free_index(&mut self) -> Option<StorageIndex> {
        let free = self.records[0].position;

        if free != 0 {
            self.records[0].position = self.records[free as usize].position;
            return Some(StorageIndex::from(free));
        }

        None
    }
}
