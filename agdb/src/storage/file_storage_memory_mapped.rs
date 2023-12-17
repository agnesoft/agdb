use super::file_storage::FileStorage;
use super::memory_storage::MemoryStorage;
use super::StorageData;
use super::StorageSlice;
use crate::DbError;

/// The default implementation of the database storage implementing
/// [`StorageData`]. It combines the [`FileStorage`] and [`MemoryStorage`]
/// leveraging the former for the persistence and the latter for performance.
/// The read operations are implemented in terms of the [`MemoryStorage`] only
/// and the write operations are implemented in terms of both [`FileStorage`]
/// and [`MemoryStorage`].
///
/// The file based storage is using write ahead logging (WAL) and single underlying
/// file. See [`FileStorage`] for more details on the persistent storage.
pub struct FileStorageMemoryMapped {
    file: FileStorage,
    memory: MemoryStorage,
}

impl StorageData for FileStorageMemoryMapped {
    fn backup(&mut self, name: &str) -> Result<(), DbError> {
        self.file.backup(name)
    }

    fn flush(&mut self) -> Result<(), DbError> {
        self.file.flush()
    }

    fn len(&self) -> u64 {
        self.file.len()
    }

    fn name(&self) -> &str {
        self.file.name()
    }

    fn new(name: &str) -> Result<Self, DbError> {
        let file = FileStorage::new(name)?;
        let buffer = file.read(0, file.len())?.to_vec();

        Ok(Self {
            file,
            memory: MemoryStorage::from_buffer(name, buffer),
        })
    }

    fn read(&self, pos: u64, value_len: u64) -> Result<StorageSlice, DbError> {
        self.memory.read(pos, value_len)
    }

    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        self.file.rename(new_name)?;
        self.memory.rename(new_name)
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        self.memory.resize(new_len)?;
        self.file.resize(new_len)
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        self.memory.write(pos, bytes)?;
        self.file.write(pos, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::write_ahead_log::WriteAheadLog;
    use crate::storage::Storage;
    use crate::storage::StorageIndex;
    use crate::test_utilities::test_file::TestFile;
    use crate::utilities::serialize::Serialize;
    use crate::utilities::serialize::SerializeStatic;

    #[test]
    fn bad_file() {
        assert!(Storage::<FileStorageMemoryMapped>::new("/a/").is_err());
    }

    #[test]
    fn index_reuse() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
    fn index_reuse_after_restore() {
        let test_file = TestFile::new();

        let index2;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

            let _index1 = storage.insert(&"Hello, World!".to_string()).unwrap();
            index2 = storage.insert(&10_i64).unwrap();
            let _index3 = storage.insert(&vec![1_u64, 2_u64, 3_u64]).unwrap();

            storage.remove(index2).unwrap();
        }

        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index4 = storage
            .insert(&vec!["Hello".to_string(), "World".to_string()])
            .unwrap();

        assert_eq!(index2, index4);
    }

    #[test]
    fn index_reuse_chain_after_restore() {
        let test_file = TestFile::new();

        let index1;
        let index2;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

            index1 = storage.insert(&"Hello, World!".to_string()).unwrap();
            index2 = storage.insert(&10_i64).unwrap();
            let _index3 = storage.insert(&vec![1_u64, 2_u64, 3_u64]).unwrap();

            storage.remove(index1).unwrap();
            storage.remove(index2).unwrap();
        }

        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index4 = storage
            .insert(&vec!["Hello".to_string(), "World".to_string()])
            .unwrap();
        let index5 = storage.insert(&1_u64).unwrap();
        let index6 = storage.insert(&vec![0_u8; 0]).unwrap();

        assert_eq!(index2, index4);
        assert_eq!(index1, index5);
        assert_eq!(index6.0, 4);
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.insert_at(StorageIndex::from(1_u64), 8, &1_i64),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn move_at_left() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.remove(StorageIndex::from(1_u64)),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn replace_larger() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let value = "Hello, World!".to_string();
        let expected_size = value.serialized_size();

        storage.replace(index, &value).unwrap();

        assert_eq!(storage.value_size(index).unwrap(), expected_size);
    }

    #[test]
    fn replace_missing_index() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.replace(StorageIndex::from(1_u64), &10_i64),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn replace_same_size() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let size = storage.value_size(index).unwrap();

        storage.replace(index, &10_i64).unwrap();

        assert_eq!(storage.value_size(index).unwrap(), size);
    }

    #[test]
    fn replace_smaller() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index = storage.insert(&"Hello, World!".to_string()).unwrap();
        let value = 1_i64;
        let expected_size = i64::serialized_size_static();

        storage.replace(index, &value).unwrap();

        assert_eq!(storage.value_size(index).unwrap(), expected_size);
    }

    #[test]
    fn resize_at_end_does_not_move() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let size = storage.len();
        let value_size = storage.value_size(index).unwrap();

        storage.resize_value(index, value_size + 8).unwrap();

        assert_eq!(storage.len(), size + 8);
    }

    #[test]
    fn resize_value_greater() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, expected_size * 2).unwrap();

        assert_eq!(storage.value_size(index), Ok(expected_size * 2));
    }

    #[test]
    fn resize_value_missing_index() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.resize_value(StorageIndex::from(1_u64), 1),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn resize_value_same() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, expected_size).unwrap();

        assert_eq!(storage.value_size(index), Ok(expected_size));
    }

    #[test]
    fn resize_value_smaller() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, expected_size / 2).unwrap();

        assert_eq!(storage.value_size(index), Ok(expected_size / 2));
    }

    #[test]
    fn resize_value_zero() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, 0).unwrap();

        assert_eq!(storage.value_size(index), Ok(0));
    }

    #[test]
    fn resize_value_resizes_file() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();

        let index;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
            index = storage.insert(&10_i64).unwrap();
            storage.insert(&5_i64).unwrap();
            storage.resize_value(index, 1).unwrap();
            storage.remove(index).unwrap();
        }

        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn restore_from_file() {
        let test_file = TestFile::new();
        let value1 = vec![1_i64, 2_i64, 3_i64];
        let value2 = 64_u64;
        let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
        let index1;
        let index2;
        let index3;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
        }

        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(index1), Ok(value1));
        assert_eq!(storage.value::<u64>(index2), Ok(value2));
        assert_eq!(storage.value::<Vec<i64>>(index3), Ok(value3));
    }

    #[test]
    fn restore_from_file_with_removed_index() {
        let test_file = TestFile::new();
        let value1 = vec![1_i64, 2_i64, 3_i64];
        let value2 = 64_u64;
        let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
        let index1;
        let index2;
        let index3;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
            storage.remove(index2).unwrap();
        }

        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(index1), Ok(value1));
        assert_eq!(
            storage.value::<u64>(StorageIndex::default()),
            Err(DbError::from("Storage error: index (0) not found"))
        );
        assert_eq!(
            storage.value::<u64>(index2),
            Err(DbError::from(format!(
                "Storage error: index ({}) not found",
                index2.0
            )))
        );
        assert_eq!(storage.value::<Vec<i64>>(index3), Ok(value3));
    }

    #[test]
    fn restore_from_file_with_all_indexes_removed() {
        let test_file = TestFile::new();
        let value1 = vec![1_i64, 2_i64, 3_i64];
        let value2 = 64_u64;
        let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
        let index1;
        let index2;
        let index3;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
            storage.remove(index1).unwrap();
            storage.remove(index2).unwrap();
            storage.remove(index3).unwrap();
        }

        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value::<u64>(StorageIndex::default()),
            Err(DbError::from("Storage error: index (0) not found"))
        );
        assert_eq!(
            storage.value::<Vec<i64>>(index1),
            Err(DbError::from(format!(
                "Storage error: index ({}) not found",
                index1.0
            )))
        );
        assert_eq!(
            storage.value::<u64>(index2),
            Err(DbError::from(format!(
                "Storage error: index ({}) not found",
                index2.0
            )))
        );
        assert_eq!(
            storage.value::<Vec<i64>>(index3),
            Err(DbError::from(format!(
                "Storage error: index ({}) not found",
                index3.0
            )))
        );
    }

    #[test]
    fn restore_from_file_with_wal() {
        let test_file = TestFile::new();
        let value1 = vec![1_i64, 2_i64, 3_i64];
        let value2 = 64_u64;
        let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
        let index1;
        let index2;
        let index3;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
        }

        let mut wal = WriteAheadLog::new(test_file.file_name()).unwrap();
        wal.insert(u64::serialized_size_static() * 2, &2_u64.serialize())
            .unwrap();

        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(index1), Ok(vec![1_i64, 2_i64]));
        assert_eq!(storage.value::<u64>(index2), Ok(value2));
        assert_eq!(storage.value::<Vec<i64>>(index3), Ok(value3));
    }

    #[test]
    fn shrink_to_fit() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();
        storage.remove(index2).unwrap();
        storage.shrink_to_fit().unwrap();

        let actual_size = std::fs::metadata(test_file.file_name()).unwrap().len();
        let expected_size =
            (u64::serialized_size_static() * 2) * 2 + i64::serialized_size_static() * 2;

        assert_eq!(actual_size, expected_size);
        assert_eq!(storage.value(index1), Ok(1_i64));
        assert_eq!(storage.value(index3), Ok(3_i64));
    }

    #[test]
    fn shrink_to_fit_no_change() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();

        let actual_size = std::fs::metadata(test_file.file_name()).unwrap().len();

        storage.shrink_to_fit().unwrap();

        assert_eq!(
            actual_size,
            std::fs::metadata(test_file.file_name()).unwrap().len()
        );
        assert_eq!(storage.value(index1), Ok(1_i64));
        assert_eq!(storage.value(index2), Ok(2_i64));
        assert_eq!(storage.value(index3), Ok(3_i64));
    }

    #[test]
    fn shrink_to_fit_uncommitted() {
        let test_file = TestFile::new();

        let expected_size;
        let index1;
        let index2;
        let index3;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&1_i64).unwrap();
            index2 = storage.insert(&2_i64).unwrap();
            index3 = storage.insert(&3_i64).unwrap();
            storage.remove(index2).unwrap();

            expected_size = std::fs::metadata(test_file.file_name()).unwrap().len();

            storage.transaction();
            storage.shrink_to_fit().unwrap();
        }

        let actual_size = std::fs::metadata(test_file.file_name()).unwrap().len();
        assert_eq!(actual_size, expected_size);

        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        assert_eq!(storage.value(index1), Ok(1_i64));
        assert_eq!(
            storage.value::<i64>(index2),
            Err(DbError::from(format!(
                "Storage error: index ({}) not found",
                index2.0
            )))
        );
        assert_eq!(storage.value(index3), Ok(3_i64));
    }

    #[test]
    fn transaction_commit() {
        let test_file = TestFile::new();
        let index;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
            let id = storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            storage.commit(id).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
        }

        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        assert_eq!(storage.value::<i64>(index), Ok(1_i64));
    }

    #[test]
    fn transaction_commit_no_transaction() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        assert_eq!(storage.commit(0), Ok(()));
    }

    #[test]
    fn transaction_unfinished() {
        let test_file = TestFile::new();
        let index;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
        }

        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
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
        let test_file = TestFile::new();
        let index;

        {
            let mut storage =
                Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
            let _ = storage.transaction();
            let id2 = storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
            storage.commit(id2).unwrap();
        }

        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(storage.value::<i64>(index), Ok(10_i64));
    }

    #[test]
    fn value_at() {
        let test_file = TestFile::new();

        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
        let data = vec![1_i64, 2_i64, 3_i64];

        let index = storage.insert(&data).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static();

        assert_eq!(storage.value_at::<i64>(index, offset), Ok(2_i64));
    }

    #[test]
    fn value_at_dynamic_size() {
        let test_file = TestFile::new();

        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();
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
        let test_file = TestFile::new();
        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value_at::<i64>(StorageIndex::from(1_u64), 8),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn value_at_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value::<i64>(StorageIndex::from(1_u64)),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }

    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

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
        let test_file = TestFile::new();
        let mut storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));
    }

    #[test]
    fn value_size_of_missing_index() {
        let test_file = TestFile::new();
        let storage = Storage::<FileStorageMemoryMapped>::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value_size(StorageIndex::from(1_u64)),
            Err(DbError::from("Storage error: index (1) not found"))
        );
    }
}
