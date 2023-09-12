use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;

#[derive(Clone, Copy, Default)]
pub struct StorageRecord {
    pub index: u64,
    pub pos: u64,
    pub size: u64,
}

impl StorageRecord {
    pub fn value_start(&self) -> u64 {
        self.pos + self.index.serialized_size() + self.size.serialized_size()
    }

    pub fn end(&self) -> u64 {
        self.value_start() + self.size
    }
}

pub struct StorageRecords {
    records: Vec<StorageRecord>,
}

impl StorageRecords {
    pub fn new() -> Self {
        Self {
            records: vec![StorageRecord::default()],
        }
    }

    pub fn new_record(&mut self, pos: u64, size: u64) -> StorageRecord {
        let record;

        if self.records[0].index != 0 {
            let index = self.records[0].index;
            self.records[0].index = self.records[index as usize].index;
            record = StorageRecord { index, pos, size };
            self.records[index as usize] = record;
        } else {
            record = StorageRecord {
                index: self.records.len() as u64,
                pos,
                size,
            };
            self.records.push(record);
        }

        record
    }

    pub fn records(&self) -> Vec<StorageRecord> {
        let mut res = Vec::<StorageRecord>::new();
        res.reserve(self.records.len());

        for record in &self.records {
            if self.is_valid(record) {
                res.push(*record);
            }
        }

        res.sort_by(|left, right| left.pos.cmp(&right.pos));

        res
    }

    pub fn set_pos(&mut self, index: u64, pos: u64) {
        if let Some(i) = self.records.get_mut(index as usize) {
            i.pos = pos;
        }
    }

    pub fn set_records(&mut self, records: Vec<StorageRecord>) {
        self.records = records;

        for index in 1..self.records.len() {
            if !self.is_valid(&self.records[index]) {
                self.remove_index(index as u64);
            }
        }
    }

    pub fn set_size(&mut self, index: u64, size: u64) {
        if let Some(i) = self.records.get_mut(index as usize) {
            i.size = size;
        }
    }

    pub fn record(&self, index: u64) -> Result<StorageRecord, DbError> {
        if let Some(record) = self.records.get(index as usize) {
            if self.is_valid(record) {
                return Ok(*record);
            }
        }

        Err(DbError::from(format!(
            "Storage error: index ({index}) not found"
        )))
    }

    pub fn remove_index(&mut self, index: u64) {
        let next_free = self.records[0].index;

        if let Some(record) = self.records.get_mut(index as usize) {
            record.index = next_free;
            record.pos = u64::MAX;
            self.records[0].index = index;
        }
    }

    fn is_valid(&self, record: &StorageRecord) -> bool {
        record.index != 0 && self.records[record.index as usize].index == record.index
    }
}
