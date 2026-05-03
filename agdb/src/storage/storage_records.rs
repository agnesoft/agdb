use crate::DbError;
use crate::utilities::serialize::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const HEADER_SIZE: u64 = 16;

#[derive(Debug, Clone, Copy, Default)]
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

#[derive(Debug, Clone)]
pub struct StorageRecords {
    records: Vec<StorageRecord>,
    free_pos_size: BTreeMap<u64, u64>,
    free_size_pos: BTreeMap<u64, BTreeSet<u64>>,
    free_size: u64,
}

impl StorageRecords {
    pub fn new() -> Self {
        Self {
            records: vec![StorageRecord::default()],
            free_pos_size: BTreeMap::new(),
            free_size_pos: BTreeMap::new(),
            free_size: 0,
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

    pub fn clear_free(&mut self) {
        self.free_pos_size.clear();
        self.free_size_pos.clear();
        self.free_size = 0;
    }

    pub fn records(&self) -> Vec<StorageRecord> {
        let mut res = Vec::with_capacity(self.records.len());

        for record in &self.records {
            if self.is_valid(record) {
                res.push(*record);
            }
        }

        res.sort_by_key(|left| left.pos);

        res
    }

    pub fn set_pos(&mut self, index: u64, pos: u64) {
        if let Some(i) = self.records.get_mut(index as usize) {
            i.pos = pos;
        }
    }

    pub fn set_record(&mut self, record: StorageRecord) {
        if record.index == 0 {
            self.mark_free(record.pos, record.size);
        } else {
            let index = record.index as usize;

            if self.records.len() <= index {
                self.records.resize(index + 1, StorageRecord::default());
            }

            self.records[index] = record;
        }
    }

    pub fn rebuild_free_index(&mut self) {
        for index in 1..self.records.len() {
            if self.records[index].index == 0 {
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
        if let Some(record) = self.records.get(index as usize)
            && self.is_valid(record)
        {
            return Ok(*record);
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

    pub fn take_free(&mut self, min_size: u64) -> Option<(u64, u64)> {
        let (&data_size, positions) = self
            .free_size_pos
            .range_mut(min_size..)
            .find(|(size, _)| **size == min_size || **size >= min_size + HEADER_SIZE)?;
        let pos = *positions.iter().next()?;
        self.remove_free(pos);

        Some((pos, data_size))
    }

    pub fn take_free_after(&mut self, end_pos: u64, min_size: u64) -> Option<(u64, u64)> {
        if let Some(size) = self
            .free_pos_size
            .get(&end_pos)
            .filter(|size| (HEADER_SIZE + **size) == min_size || **size >= min_size + HEADER_SIZE)
            .cloned()
        {
            self.remove_free(end_pos);
            return Some((end_pos, size));
        }

        None
    }

    pub fn mark_free_compact(&mut self, pos: u64, size: u64) -> u64 {
        let mut end_pos = pos + HEADER_SIZE + size;
        let mut size = size;

        while let Some(next_size) = self.free_pos_size.get(&end_pos).cloned() {
            self.remove_free(end_pos);
            end_pos += HEADER_SIZE + next_size;
            size += HEADER_SIZE + next_size;
        }

        self.mark_free(pos, size);
        size
    }

    #[allow(dead_code)]
    pub fn free_size(&self) -> u64 {
        self.free_size
    }

    fn mark_free(&mut self, pos: u64, size: u64) {
        self.free_pos_size.insert(pos, size);
        self.free_size_pos.entry(size).or_default().insert(pos);
        self.free_size += size;
    }

    fn remove_free(&mut self, pos: u64) {
        let size = self.free_pos_size.remove(&pos).unwrap_or_default();

        if let Some(positions) = self.free_size_pos.get_mut(&size) {
            positions.remove(&pos);

            if positions.is_empty() {
                self.free_size_pos.remove(&size);
            }
        }

        self.free_size -= size;
    }

    fn is_valid(&self, record: &StorageRecord) -> bool {
        record.index != 0 && self.records[record.index as usize].index == record.index
    }
}
