use super::StorageData;
use super::StorageSlice;
use crate::DbError;

pub struct MemoryStorage {
    buffer: Vec<u8>,
    name: String,
}

/// In memory buffer equivalent to `Vec<u8>` implementing [`StorageData`].
/// The [`StorageData::read()`] always returns non-owning the slice to the
/// underlying buffer. The [`StorageData::backup()`] and [`StorageData::flush()`]
/// rely on default implementations that do nothing.
///
/// This implementation offers unmatched performance but also **no persistence**.
impl MemoryStorage {
    pub fn from_buffer(name: &str, buffer: Vec<u8>) -> Self {
        Self {
            buffer,
            name: name.to_string(),
        }
    }
}

impl StorageData for MemoryStorage {
    fn copy(&self, name: &str) -> Result<Self, DbError> {
        Ok(Self {
            buffer: self.buffer.clone(),
            name: name.to_string(),
        })
    }

    fn len(&self) -> u64 {
        self.buffer.len() as u64
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn new(name: &str) -> Result<Self, DbError> {
        Ok(Self {
            buffer: vec![],
            name: name.to_string(),
        })
    }

    fn read(&self, pos: u64, value_len: u64) -> Result<StorageSlice, DbError> {
        let end = pos + value_len;
        Ok(StorageSlice::from(&self.buffer[pos as usize..end as usize]))
    }

    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        self.name = new_name.to_string();
        Ok(())
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        self.buffer.resize(new_len as usize, 0);
        Ok(())
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        let current_len = self.len();
        let end = pos + bytes.len() as u64;

        if end < current_len {
            self.buffer[pos as usize..end as usize].copy_from_slice(bytes);
        } else {
            self.buffer.resize(pos as usize, 0);
            self.buffer.extend_from_slice(bytes);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use crate::storage::StorageIndex;
    use crate::utilities::serialize::Serialize;
    use crate::utilities::serialize::SerializeStatic;

    #[test]
    fn backup_does_nothing() {
        let storage = Storage::<MemoryStorage>::new("storage").unwrap();
        assert!(storage.backup("storage2").is_ok())
    }

    #[test]
    fn name() {
        let storage = Storage::<MemoryStorage>::new("storage").unwrap();
        assert_eq!(storage.name(), "storage");
    }

    #[test]
    fn is_empty() {
        let mut store = MemoryStorage::new("name").unwrap();
        assert!(store.is_empty());
        store.write(0, "Hi".as_bytes()).unwrap();
        assert!(!store.is_empty());
    }

    #[test]
    fn index_reuse() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let _index1 = storage.insert(&"Hello, World!".to_string()).unwrap();
        let index2 = storage.insert(&10_i64).unwrap();
        let _index3 = storage.insert(&vec![1_u64, 2_u64, 3_u64]).unwrap();

        storage.remove(index2).unwrap();

        let index4 = storage
            .insert(&vec!["Hello".to_string(), "World".to_string()])
            .unwrap();

        assert_eq!(index2, index4);
    }

    #[test]
    fn insert() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let value1 = "Hello, World!".to_string();
        let index1 = storage.insert(&value1).unwrap();
        assert_eq!(storage.value_size(index1), Ok(value1.serialized_size()));
        assert_eq!(storage.value_size(index1), Ok(value1.serialized_size()));
        assert_eq!(storage.value(index1), Ok(value1));

        let value2 = 10_i64;
        let index2 = storage.insert(&value2).unwrap();
        assert_eq!(
            storage.value_size(index2),
            Ok(i64::serialized_size_static())
        );
        assert_eq!(
            storage.value_size(index2),
            Ok(i64::serialized_size_static())
        );
        assert_eq!(storage.value(index2), Ok(value2));

        let value3 = vec![1_u64, 2_u64, 3_u64];
        let index3 = storage.insert(&value3).unwrap();
        assert_eq!(storage.value_size(index3), Ok(value3.serialized_size()));
        assert_eq!(storage.value_size(index3), Ok(value3.serialized_size()));
        assert_eq!(storage.value(index3), Ok(value3));

        let value4 = vec!["Hello".to_string(), "World".to_string()];
        let index4 = storage.insert(&value4).unwrap();
        assert_eq!(storage.value_size(index4), Ok(value4.serialized_size()));
        assert_eq!(storage.value_size(index4), Ok(value4.serialized_size()));
        assert_eq!(storage.value(index4), Ok(value4));
    }

    #[test]
    fn insert_at() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static();

        storage.insert_at(index, offset, &10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 10_i64, 3_i64]
        );
    }

    #[test]
    fn insert_at_value_end() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static() * 3;

        storage.insert_at(index, 0, &4_u64).unwrap();
        storage.insert_at(index, offset, &10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_value_end_multiple_values() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        storage.insert(&"Hello, World!".to_string()).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static() * 3;

        storage.insert_at(index, 0, &4_u64).unwrap();
        storage.insert_at(index, offset, &10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_beyond_end() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static() * 4;

        storage.insert_at(index, 0, &5_u64).unwrap();
        storage.insert_at(index, offset, &10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 0_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_beyond_end_multiple_values() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        storage.insert(&"Hello, World!".to_string()).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static() * 4;

        storage.insert_at(index, 0, &5_u64).unwrap();
        storage.insert_at(index, offset, &10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 0_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_missing_index() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        assert_eq!(
            storage.insert_at(StorageIndex::from(1_u64), 8, &1_i64),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn move_at_left() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = u64::serialized_size_static() + i64::serialized_size_static() * 2;
        let offset_to = u64::serialized_size_static() + i64::serialized_size_static();
        let size = i64::serialized_size_static();

        storage
            .move_at(index, offset_from, offset_to, size)
            .unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 3_i64, 0_i64]
        )
    }

    #[test]
    fn move_at_left_overlapping() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = u64::serialized_size_static() + i64::serialized_size_static();
        let offset_to = u64::serialized_size_static();
        let size = u64::serialized_size_static() * 2;

        storage
            .move_at(index, offset_from, offset_to, size)
            .unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![2_i64, 3_i64, 0_i64]
        )
    }

    #[test]
    fn move_at_right() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = u64::serialized_size_static() + i64::serialized_size_static();
        let offset_to = u64::serialized_size_static() + i64::serialized_size_static() * 2;
        let size = u64::serialized_size_static();

        storage
            .move_at(index, offset_from, offset_to, size)
            .unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 0_i64, 2_i64]
        )
    }

    #[test]
    fn move_at_right_overlapping() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = u64::serialized_size_static();
        let offset_to = u64::serialized_size_static() + i64::serialized_size_static();
        let size = u64::serialized_size_static() * 2;

        storage
            .move_at(index, offset_from, offset_to, size)
            .unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![0_i64, 1_i64, 2_i64]
        )
    }

    #[test]
    fn move_at_beyond_end() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = u64::serialized_size_static() + i64::serialized_size_static();
        let offset_to = u64::serialized_size_static() + i64::serialized_size_static() * 4;
        let size = u64::serialized_size_static();

        storage
            .move_at(index, offset_from, offset_to, size)
            .unwrap();

        storage.insert_at(index, 0, &5_u64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 0_i64, 3_i64, 0_i64, 2_i64]
        )
    }

    #[test]
    fn move_at_size_out_of_bounds() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();

        assert_eq!(
            storage.move_at(index, 8, 16, 1000),
            Err(DbError::from(
                "Storage error: value size (1008) out of bounds (32)"
            ))
        )
    }

    #[test]
    fn move_at_same_offset() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = u64::serialized_size_static();
        let offset_to = u64::serialized_size_static();
        let size = u64::serialized_size_static();

        storage
            .move_at(index, offset_from, offset_to, size)
            .unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 2_i64, 3_i64]
        )
    }

    #[test]
    fn move_at_zero_size() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let value = vec![1_i64, 2_i64, 3_i64];
        let index = storage.insert(&value).unwrap();

        storage.move_at(index, 0, 1, 0).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 2_i64, 3_i64]
        );
    }

    #[test]
    fn remove() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&1_i64).unwrap();

        assert_eq!(storage.value::<i64>(index).unwrap(), 1_i64);

        storage.remove(index).unwrap();

        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn remove_missing_index() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        assert_eq!(
            storage.remove(StorageIndex::from(1_u64)),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn replace_larger() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let value = "Hello, World!".to_string();
        let expected_size = value.serialized_size();

        storage.replace(index, &value).unwrap();

        assert_eq!(storage.value_size(index).unwrap(), expected_size);
    }

    #[test]
    fn replace_missing_index() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        assert_eq!(
            storage.replace(StorageIndex::from(1_u64), &10_i64),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn replace_same_size() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let size = storage.value_size(index).unwrap();

        storage.replace(index, &10_i64).unwrap();

        assert_eq!(storage.value_size(index).unwrap(), size);
    }

    #[test]
    fn replace_smaller() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&"Hello, World!".to_string()).unwrap();
        let value = 1_i64;
        let expected_size = i64::serialized_size_static();

        storage.replace(index, &value).unwrap();

        assert_eq!(storage.value_size(index).unwrap(), expected_size);
    }

    #[test]
    fn resize_at_end_does_not_move() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let size = storage.len();
        let value_size = storage.value_size(index).unwrap();

        storage.resize_value(index, value_size + 8).unwrap();

        assert_eq!(storage.len(), size + 8);
    }

    #[test]
    fn resize_value_greater() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, expected_size * 2).unwrap();

        assert_eq!(storage.value_size(index), Ok(expected_size * 2));
    }

    #[test]
    fn resize_value_missing_index() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        assert_eq!(
            storage.resize_value(StorageIndex::from(1_u64), 1),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn resize_value_same() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, expected_size).unwrap();

        assert_eq!(storage.value_size(index), Ok(expected_size));
    }

    #[test]
    fn resize_value_smaller() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, expected_size / 2).unwrap();

        assert_eq!(storage.value_size(index), Ok(expected_size / 2));
    }

    #[test]
    fn resize_value_zero() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, 0).unwrap();

        assert_eq!(storage.value_size(index), Ok(0));
    }

    #[test]
    fn resize_value_resizes_storage() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&3_i64).unwrap();
        let len = storage.len();
        let size = u64::serialized_size_static() + i64::serialized_size_static() * 3;
        let expected_len = len + i64::serialized_size_static() * 3;

        storage.resize_value(index, size).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(index).unwrap(), vec![0_i64; 3]);
        assert_eq!(storage.len(), expected_len);
    }

    #[test]
    fn resize_value_invalidates_original_position() {
        let index;

        {
            let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();
            index = storage.insert(&10_i64).unwrap();
            storage.insert(&5_i64).unwrap();
            storage.resize_value(index, 1).unwrap();
            storage.remove(index).unwrap();
        }

        let storage = Storage::<MemoryStorage>::new("storage").unwrap();

        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn shrink_to_fit() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();
        storage.remove(index2).unwrap();
        storage.shrink_to_fit().unwrap();

        let expected_size =
            (u64::serialized_size_static() * 2) * 3 + i64::serialized_size_static() * 3;

        assert_eq!(storage.len(), expected_size);
        assert_eq!(storage.value(index1), Ok(1_i64));
        assert_eq!(storage.value(index3), Ok(3_i64));
    }

    #[test]
    fn shrink_to_fit_no_change() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();

        storage.shrink_to_fit().unwrap();

        assert_eq!(storage.value(index1), Ok(1_i64));
        assert_eq!(storage.value(index2), Ok(2_i64));
        assert_eq!(storage.value(index3), Ok(3_i64));
    }

    #[test]
    fn transaction_commit() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();
        let id = storage.transaction();
        let index = storage.insert(&1_i64).unwrap();
        storage.commit(id).unwrap();
        assert_eq!(storage.value::<i64>(index), Ok(1_i64));
    }

    #[test]
    fn transaction_commit_no_transaction() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();
        assert_eq!(storage.commit(0), Ok(()));
    }

    #[test]
    fn transaction_unfinished() {
        let index;

        {
            let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
        }

        let storage = Storage::<MemoryStorage>::new("storage").unwrap();
        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::from(format!(
                "Storage error: index ({}) not found",
                index.0
            )))
        );
    }

    #[test]
    fn transaction_nested_unfinished() {
        let index;

        {
            let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();
            let _ = storage.transaction();
            let id2 = storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
            storage.commit(id2).unwrap();
        }

        let storage = Storage::<MemoryStorage>::new("storage").unwrap();
        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::from(format!(
                "Storage error: index ({}) not found",
                index.0
            )))
        );
    }

    #[test]
    fn transaction_commit_mismatch() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();
        let id1 = storage.transaction();
        let id2 = storage.transaction();
        let index = storage.insert(&1_i64).unwrap();
        assert_eq!(storage.value::<i64>(index), Ok(1_i64));

        assert_eq!(
            storage.commit(id1),
            Err(DbError::from(format!(
                "Cannot end transaction '{id1}'. Transaction '{id2}' in progress."
            )))
        );
    }

    #[test]
    fn value() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();
        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(storage.value::<i64>(index), Ok(10_i64));
    }

    #[test]
    fn value_at() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();
        let data = vec![1_i64, 2_i64, 3_i64];

        let index = storage.insert(&data).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static();

        assert_eq!(storage.value_at::<i64>(index, offset), Ok(2_i64));
    }

    #[test]
    fn value_at_dynamic_size() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();
        let data = vec![2_i64, 1_i64, 2_i64];

        let index = storage.insert(&data).unwrap();
        let offset = u64::serialized_size_static();

        assert_eq!(
            storage.value_at::<Vec<i64>>(index, offset),
            Ok(vec![1_i64, 2_i64])
        );
    }

    #[test]
    fn value_at_of_missing_index() {
        let storage = Storage::<MemoryStorage>::new("storage").unwrap();

        assert_eq!(
            storage.value_at::<i64>(StorageIndex::from(1_u64), 8),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn value_at_out_of_bounds() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let data = vec![1_i64, 2_i64];
        let index = storage.insert(&data).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static() * 2;

        assert_eq!(
            storage.value_at::<i64>(index, offset),
            Err(DbError::from("i64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn value_at_offset_overflow() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let data = vec![1_i64, 2_i64];
        let index = storage.insert(&data).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static() * 3;

        assert_eq!(
            storage.value_at::<i64>(index, offset),
            Err(DbError::from(
                "Storage error: offset (32) out of bounds (24)"
            ))
        );
    }

    #[test]
    fn value_of_missing_index() {
        let storage = Storage::<MemoryStorage>::new("storage").unwrap();

        assert_eq!(
            storage.value::<i64>(StorageIndex::from(1_u64)),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn value_out_of_bounds() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index),
            Err(DbError::from(
                "Vec<i64> deserialization error: out of bounds"
            ))
        );
    }

    #[test]
    fn value_size() {
        let mut storage = Storage::<MemoryStorage>::new("storage").unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));
    }

    #[test]
    fn value_size_of_missing_index() {
        let storage = Storage::<MemoryStorage>::new("storage").unwrap();

        assert_eq!(
            storage.value_size(StorageIndex::from(1_u64)),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }
}
