use crate::DbError;
use crate::DbErrorType;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub(crate) const STORAGE_RECORD_SIZE: u64 = 16; // 8 bytes for index + 8 bytes for size

#[derive(Debug, Clone, Copy, Default)]
pub struct StorageRecord {
    pub index: u64,
    pub pos: u64,
    pub size: u64,
}

impl StorageRecord {
    pub fn value_start(&self) -> u64 {
        self.pos + STORAGE_RECORD_SIZE
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

        Err(DbError::storage(
            DbErrorType::NotFound,
            format!("Index ({index}) not found"),
        ))
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
        let (pos, data_size) = self
            .free_size_pos
            .range(min_size..)
            .filter(|(size, _)| **size == min_size || **size >= min_size + STORAGE_RECORD_SIZE)
            .find_map(|(size, positions)| positions.first().map(|pos| (*pos, *size)))?;
        self.remove_free(pos);

        Some((pos, data_size))
    }

    pub fn take_free_after(&mut self, end_pos: u64, min_size: u64) -> Option<(u64, u64)> {
        if let Some(size) = self
            .free_pos_size
            .get(&end_pos)
            .filter(|size| (STORAGE_RECORD_SIZE + **size) == min_size || **size >= min_size)
            .cloned()
        {
            self.remove_free(end_pos);
            return Some((end_pos, size));
        }

        None
    }

    pub fn mark_free_compact(&mut self, pos: u64, size: u64) -> (u64, u64) {
        let mut pos = pos;
        let mut end_pos = pos + STORAGE_RECORD_SIZE + size;

        while let Some(next_size) = self.free_pos_size.get(&end_pos).cloned() {
            self.remove_free(end_pos);
            end_pos += STORAGE_RECORD_SIZE + next_size;
        }

        while let Some((prev_pos, _)) = self
            .free_pos_size
            .range(..pos)
            .next_back()
            .filter(|(prev_pos, size)| *prev_pos + STORAGE_RECORD_SIZE + *size == pos)
            .map(|(prev_pos, size)| (*prev_pos, *size))
        {
            self.remove_free(prev_pos);
            pos = prev_pos;
        }

        let size = end_pos - pos - STORAGE_RECORD_SIZE;
        self.mark_free(pos, size);
        (pos, size)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn free_regions(regions: &[(u64, u64)]) -> StorageRecords {
        let mut records = StorageRecords::new();

        for &(pos, size) in regions {
            records.set_record(StorageRecord {
                index: 0,
                pos,
                size,
            });
        }

        records
    }

    fn not_found(index: u64) -> DbError {
        DbError::storage(DbErrorType::NotFound, format!("Index ({index}) not found"))
    }

    mod storage_record {
        use super::*;

        mod value_start {
            use super::*;

            #[test]
            fn value_start_should_follow_the_header() {
                let record = StorageRecord {
                    index: 1,
                    pos: 100,
                    size: 10,
                };

                assert_eq!(record.value_start(), 100 + STORAGE_RECORD_SIZE);
            }
        }

        mod end {
            use super::*;

            #[test]
            fn end_should_follow_the_value() {
                let record = StorageRecord {
                    index: 1,
                    pos: 100,
                    size: 10,
                };

                assert_eq!(record.end(), 100 + STORAGE_RECORD_SIZE + 10);
            }
        }
    }

    mod storage_records {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_have_no_records_and_no_free_regions() {
                let mut records = StorageRecords::new();

                assert!(records.records().is_empty());
                assert_eq!(records.free_size(), 0);
                assert_eq!(records.take_free(0), None);
            }
        }

        mod new_record {
            use super::*;

            #[test]
            fn new_record_should_assign_next_index() {
                let mut records = StorageRecords::new();

                let first = records.new_record(24, 8);
                let second = records.new_record(48, 16);

                assert_eq!((first.index, first.pos, first.size), (1, 24, 8));
                assert_eq!((second.index, second.pos, second.size), (2, 48, 16));
            }

            #[test]
            fn new_record_should_reuse_last_removed_index_first() {
                let mut records = StorageRecords::new();
                records.new_record(0, 0);
                records.new_record(0, 0);
                records.new_record(0, 0);
                records.remove_index(1);
                records.remove_index(3);

                let indexes = [0, 0, 0].map(|_| records.new_record(0, 0).index);

                assert_eq!(indexes, [3, 1, 4]);
            }
        }

        mod clear_free {
            use super::*;

            #[test]
            fn clear_free_should_forget_free_regions() {
                let mut records = free_regions(&[(100, 10), (200, 20)]);

                records.clear_free();

                assert_eq!(records.free_size(), 0);
                assert_eq!(records.take_free(0), None);
            }
        }

        mod records {
            use super::*;

            #[test]
            fn records_should_return_valid_records_sorted_by_pos() {
                let mut records = StorageRecords::new();
                records.new_record(100, 1);
                records.new_record(50, 2);
                records.new_record(10, 3);
                records.remove_index(2);

                let positions: Vec<(u64, u64)> = records
                    .records()
                    .iter()
                    .map(|record| (record.index, record.pos))
                    .collect();

                assert_eq!(positions, vec![(3, 10), (1, 100)]);
            }
        }

        mod set_pos {
            use super::*;

            #[test]
            fn set_pos_should_move_the_record() {
                let mut records = StorageRecords::new();
                let record = records.new_record(24, 8);

                records.set_pos(record.index, 64);

                assert_eq!(records.record(record.index).unwrap().pos, 64);
            }

            #[test]
            fn set_pos_missing_index_should_do_nothing() {
                let mut records = StorageRecords::new();
                records.new_record(24, 8);

                records.set_pos(10, 64);

                assert_eq!(records.records().len(), 1);
                assert_eq!(records.record(1).unwrap().pos, 24);
            }
        }

        mod set_record {
            use super::*;

            #[test]
            fn set_record_should_extend_the_records() {
                let mut records = StorageRecords::new();

                records.set_record(StorageRecord {
                    index: 3,
                    pos: 40,
                    size: 8,
                });

                assert_eq!(records.record(3).unwrap().pos, 40);
                assert_eq!(records.record(1).unwrap_err(), not_found(1));
                assert_eq!(records.record(2).unwrap_err(), not_found(2));
            }

            #[test]
            fn set_record_existing_index_should_replace_the_record() {
                let mut records = StorageRecords::new();
                records.new_record(24, 8);

                records.set_record(StorageRecord {
                    index: 1,
                    pos: 64,
                    size: 16,
                });

                let record = records.record(1).unwrap();
                assert_eq!((record.pos, record.size), (64, 16));
            }

            #[test]
            fn set_record_index_zero_should_mark_free_region() {
                let mut records = StorageRecords::new();

                records.set_record(StorageRecord {
                    index: 0,
                    pos: 40,
                    size: 8,
                });

                assert!(records.records().is_empty());
                assert_eq!(records.free_size(), 8);
                assert_eq!(records.take_free(8), Some((40, 8)));
            }
        }

        mod rebuild_free_index {
            use super::*;

            #[test]
            fn rebuild_free_index_should_make_missing_indexes_reusable() {
                let mut records = StorageRecords::new();
                records.set_record(StorageRecord {
                    index: 3,
                    pos: 40,
                    size: 8,
                });

                records.rebuild_free_index();

                let indexes = [0, 0, 0].map(|_| records.new_record(0, 0).index);
                assert_eq!(indexes, [2, 1, 4]);
            }
        }

        mod set_size {
            use super::*;

            #[test]
            fn set_size_should_resize_the_record() {
                let mut records = StorageRecords::new();
                let record = records.new_record(24, 8);

                records.set_size(record.index, 32);

                assert_eq!(records.record(record.index).unwrap().size, 32);
            }

            #[test]
            fn set_size_missing_index_should_do_nothing() {
                let mut records = StorageRecords::new();
                records.new_record(24, 8);

                records.set_size(10, 32);

                assert_eq!(records.records().len(), 1);
                assert_eq!(records.record(1).unwrap().size, 8);
            }
        }

        mod record {
            use super::*;

            #[test]
            fn record_should_return_the_record() {
                let mut records = StorageRecords::new();
                records.new_record(24, 8);

                let record = records.record(1).unwrap();

                assert_eq!((record.index, record.pos, record.size), (1, 24, 8));
            }

            #[test]
            fn record_invalid_index_should_fail_with_not_found() {
                let mut records = StorageRecords::new();
                records.new_record(24, 8);
                records.new_record(32, 8);
                records.remove_index(2);

                for index in [0, 2, 3] {
                    assert_eq!(
                        records.record(index).unwrap_err(),
                        not_found(index),
                        "index: {index}"
                    );
                }
            }
        }

        mod remove_index {
            use super::*;

            #[test]
            fn remove_index_should_invalidate_the_record() {
                let mut records = StorageRecords::new();
                records.new_record(24, 8);

                records.remove_index(1);

                assert_eq!(records.record(1).unwrap_err(), not_found(1));
                assert!(records.records().is_empty());
            }

            #[test]
            fn remove_index_missing_index_should_do_nothing() {
                let mut records = StorageRecords::new();
                records.new_record(24, 8);

                records.remove_index(10);

                assert_eq!(records.records().len(), 1);
                assert_eq!(records.new_record(0, 0).index, 2);
            }
        }

        mod take_free {
            use super::*;

            #[test]
            fn take_free_should_take_region_of_exact_size() {
                let mut records = free_regions(&[(100, 40), (200, 20)]);

                assert_eq!(records.take_free(20), Some((200, 20)));
                assert_eq!(records.free_size(), 40);
            }

            #[test]
            fn take_free_should_take_smallest_region_with_room_for_another_header() {
                let mut records = free_regions(&[(100, 100), (200, 40), (300, 20)]);

                assert_eq!(records.take_free(10), Some((200, 40)));
            }

            #[test]
            fn take_free_should_take_lowest_position_of_equal_regions() {
                let mut records = free_regions(&[(300, 20), (100, 20)]);

                assert_eq!(records.take_free(20), Some((100, 20)));
            }

            #[test]
            fn take_free_too_small_regions_should_return_none() {
                let mut records = free_regions(&[(100, 10), (200, 20)]);

                assert_eq!(records.take_free(11), None);
                assert_eq!(records.free_size(), 30);
            }
        }

        mod take_free_after {
            use super::*;

            #[test]
            fn take_free_after_should_take_region_at_end_pos() {
                let mut records = free_regions(&[(100, 10)]);

                assert_eq!(records.take_free_after(100, 10), Some((100, 10)));
                assert_eq!(records.free_size(), 0);
            }

            #[test]
            fn take_free_after_should_take_region_absorbing_its_header() {
                let mut records = free_regions(&[(100, 10)]);

                assert_eq!(
                    records.take_free_after(100, STORAGE_RECORD_SIZE + 10),
                    Some((100, 10))
                );
            }

            #[test]
            fn take_free_after_unusable_region_should_return_none() {
                let mut records = free_regions(&[(100, 10)]);

                for (end_pos, min_size) in [(100, 11), (100, STORAGE_RECORD_SIZE + 11), (50, 1)] {
                    assert_eq!(
                        records.take_free_after(end_pos, min_size),
                        None,
                        "end_pos: {end_pos}, min_size: {min_size}"
                    );
                }

                assert_eq!(records.free_size(), 10);
            }
        }

        mod mark_free_compact {
            use super::*;

            #[test]
            fn mark_free_compact_should_merge_following_regions() {
                let mut records = free_regions(&[(132, 10), (158, 4)]);

                assert_eq!(records.mark_free_compact(100, 16), (100, 62));
                assert_eq!(records.take_free(62), Some((100, 62)));
                assert_eq!(records.free_size(), 0);
            }

            #[test]
            fn mark_free_compact_should_merge_preceding_regions() {
                let mut records = free_regions(&[(100, 0), (116, 16)]);

                assert_eq!(records.mark_free_compact(148, 10), (100, 58));
                assert_eq!(records.take_free(58), Some((100, 58)));
                assert_eq!(records.free_size(), 0);
            }

            #[test]
            fn mark_free_compact_should_merge_regions_on_both_sides() {
                let mut records = free_regions(&[(100, 16), (158, 4)]);

                assert_eq!(records.mark_free_compact(132, 10), (100, 62));
                assert_eq!(records.free_size(), 62);
            }

            #[test]
            fn mark_free_compact_should_not_merge_regions_not_adjacent() {
                let mut records = free_regions(&[(100, 16), (180, 4)]);

                assert_eq!(records.mark_free_compact(140, 4), (140, 4));
                assert_eq!(records.free_size(), 24);
            }
        }

        mod free_size {
            use super::*;

            #[test]
            fn free_size_should_sum_free_regions_without_headers() {
                let records = free_regions(&[(0, 10), (100, 20)]);

                assert_eq!(records.free_size(), 30);
            }
        }

        mod remove_free {
            use super::*;

            #[test]
            fn remove_free_not_free_pos_should_do_nothing() {
                let mut records = free_regions(&[(100, 10)]);

                records.remove_free(50);

                assert_eq!(records.free_size(), 10);
                assert_eq!(records.take_free(10), Some((100, 10)));
            }
        }
    }
}
