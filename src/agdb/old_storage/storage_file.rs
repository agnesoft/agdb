use crate::db::db_error::DbError;
use crate::old_storage::storage_data_file::StorageDataFile;
use crate::old_storage::storage_impl::StorageImpl;

pub type StorageFile = StorageImpl<StorageDataFile>;

impl TryFrom<String> for StorageFile {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        let mut storage = StorageFile {
            data: StorageDataFile::try_from(filename)?,
        };

        storage.apply_wal()?;
        storage.read_records()?;

        Ok(storage)
    }
}

impl TryFrom<&str> for StorageFile {
    type Error = DbError;

    fn try_from(filename: &str) -> Result<Self, Self::Error> {
        Self::try_from(filename.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::old_storage::storage_index::StorageIndex;
    use crate::old_storage::OldStorage;
    use crate::test_utilities::test_file::TestFile;
    use crate::utilities::old_serialize::OldSerialize;
    use std::fs::metadata;

    #[test]
    fn bad_file() {
        assert!(StorageFile::try_from("/a/").is_err());
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&10_i64);

        assert_eq!(index, Ok(StorageIndex::from(1_i64)));
    }

    #[test]
    fn insert_at() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = (u64::fixed_size() + i64::fixed_size()) as u64;
        storage.insert_at(&index, offset, &10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 10_i64, 3_i64]
        );
    }

    #[test]
    fn insert_at_missing_index() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        assert_eq!(
            storage.insert_at(&StorageIndex::from(1_i64), 8, &1_i64),
            Err(DbError::from("index '1' not found"))
        );
    }

    #[test]
    fn insert_at_value_end() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = (u64::fixed_size() + i64::fixed_size() * 3) as u64;
        assert_eq!(storage.insert_at(&index, 0, &4_u64).unwrap(), 8);
        assert_eq!(storage.insert_at(&index, offset, &10_i64).unwrap(), 8);

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_beyond_end() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = (u64::fixed_size() + i64::fixed_size() * 4) as u64;
        assert_eq!(storage.insert_at(&index, 0, &5_u64).unwrap(), 8);
        assert_eq!(storage.insert_at(&index, offset, &10_i64).unwrap(), 8);

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 0_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_bytes() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = (u64::fixed_size() + i64::fixed_size()) as u64;
        let size = i64::fixed_size() * 2;

        assert_eq!(
            storage
                .insert_at(&index, offset, &vec![0_u8; size as usize])
                .unwrap(),
            size
        );

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 0_i64, 0_i64]
        );
    }

    #[test]
    fn move_at() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = (u64::fixed_size() + i64::fixed_size() * 2) as u64;
        let offset_to = (u64::fixed_size() + i64::fixed_size()) as u64;
        let size = u64::fixed_size();

        storage
            .move_at(&index, offset_from, offset_to, size)
            .unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 3_i64, 0_i64]
        )
    }

    #[test]
    fn move_at_beyond_end() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = (u64::fixed_size() + i64::fixed_size()) as u64;
        let offset_to = (u64::fixed_size() + i64::fixed_size() * 4) as u64;
        let size = u64::fixed_size();

        storage
            .move_at(&index, offset_from, offset_to, size)
            .unwrap();

        storage.insert_at(&index, 0, &5_u64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 0_i64, 3_i64, 0_i64, 2_i64]
        )
    }

    #[test]
    fn move_at_missing_index() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        assert_eq!(
            storage.move_at(&StorageIndex::from(1_i64), 0, 1, 10),
            Err(DbError::from("index '1' not found"))
        );
    }

    #[test]
    fn move_at_same_offset() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();

        assert_eq!(storage.move_at(&index, 0, 0, 10), Ok(()));
        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64]
        );
    }

    #[test]
    fn move_at_size_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = (u64::fixed_size() + i64::fixed_size() * 3) as u64;
        let offset_to = (u64::fixed_size() + i64::fixed_size() * 2) as u64;
        let size = (u64::fixed_size() * 10) as u64;

        assert_eq!(
            storage.move_at(&index, offset_from, offset_to, size),
            Err(DbError::from("move size out of bounds"))
        );
    }

    #[test]
    fn move_at_zero_size() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();

        assert_eq!(storage.move_at(&index, 0, 1, 0), Ok(()));
        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64]
        );
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        storage.remove(&index).unwrap();

        assert_eq!(
            storage.value::<i64>(&index),
            Err(DbError::from("index '1' not found"))
        );
    }

    #[test]
    fn remove_missing_index() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        assert_eq!(
            storage.remove(&StorageIndex::from(1_i64)),
            Err(DbError::from("index '1' not found"))
        );
    }

    #[test]
    fn resize_at_end_does_not_move() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let size = storage.size().unwrap();
        let value_size = storage.value_size(&index).unwrap();

        storage.resize_value(&index, value_size + 8).unwrap();

        assert_eq!(storage.size(), Ok(size + 8));
    }

    #[test]
    fn resize_value_greater() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::fixed_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));

        storage.resize_value(&index, expected_size * 2).unwrap();

        assert_eq!(storage.value_size(&index), Ok(expected_size * 2));
    }

    #[test]
    fn resize_value_missing_index() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.resize_value(&StorageIndex::from(1_i64), 1),
            Err(DbError::from("index '1' not found"))
        );
    }

    #[test]
    fn resize_value_same() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::fixed_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));

        storage.resize_value(&index, expected_size).unwrap();

        assert_eq!(storage.value_size(&index), Ok(expected_size));
    }

    #[test]
    fn resize_value_smaller() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::fixed_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));

        storage.resize_value(&index, expected_size / 2).unwrap();

        assert_eq!(storage.value_size(&index), Ok(expected_size / 2));
    }

    #[test]
    fn resize_value_zero() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::fixed_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));

        assert_eq!(
            storage.resize_value(&index, 0),
            Err(DbError::from("value size cannot be 0"))
        );
    }

    #[test]
    fn resize_value_resizes_file() {
        let test_file = TestFile::new();

        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();
        let index = storage.insert(&3_i64).unwrap();
        let size = u64::fixed_size() + i64::fixed_size() * 3;
        storage.resize_value(&index, size).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(&index), Ok(vec![0_i64; 3]));
    }

    #[test]
    fn resize_value_invalidates_original_position() {
        let test_file = TestFile::new();

        let index;

        {
            let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();
            index = storage.insert(&10_i64).unwrap();
            storage.insert(&5_i64).unwrap();
            storage.resize_value(&index, 1).unwrap();
            storage.remove(&index).unwrap();
        }

        let mut storage = StorageFile::try_from(test_file.file_name().as_str()).unwrap();

        assert_eq!(
            storage.value::<i64>(&index),
            Err(DbError::from("index '1' not found"))
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
            let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
        }

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(&index1), Ok(value1));
        assert_eq!(storage.value::<u64>(&index2), Ok(value2));
        assert_eq!(storage.value::<Vec<i64>>(&index3), Ok(value3));
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
            let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
            storage.remove(&index2).unwrap();
        }

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(&index1), Ok(value1));
        assert_eq!(
            storage.value::<u64>(&StorageIndex::default()),
            Err(DbError::from(format!("index '{}' not found", 0)))
        );
        assert_eq!(
            storage.value::<u64>(&index2),
            Err(DbError::from(format!(
                "index '{}' not found",
                index2.value()
            )))
        );
        assert_eq!(storage.value::<Vec<i64>>(&index3), Ok(value3));
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
            let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
            storage.remove(&index1).unwrap();
            storage.remove(&index2).unwrap();
            storage.remove(&index3).unwrap();
        }

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.value::<u64>(&StorageIndex::default()),
            Err(DbError::from(format!("index '{}' not found", 0)))
        );
        assert_eq!(
            storage.value::<Vec<i64>>(&index1),
            Err(DbError::from(format!(
                "index '{}' not found",
                index1.value()
            )))
        );
        assert_eq!(
            storage.value::<u64>(&index2),
            Err(DbError::from(format!(
                "index '{}' not found",
                index2.value()
            )))
        );
        assert_eq!(
            storage.value::<Vec<i64>>(&index3),
            Err(DbError::from(format!(
                "index '{}' not found",
                index3.value()
            )))
        );
    }

    #[test]
    fn shrink_to_fit() {
        let test_file = TestFile::new();

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();
        storage.remove(&index2).unwrap();
        storage.shrink_to_fit().unwrap();

        let actual_size = metadata(test_file.file_name()).unwrap().len();
        let expected_size = (u64::fixed_size() * 2) * 2 + i64::fixed_size() * 2;

        assert_eq!(actual_size, expected_size);
        assert_eq!(storage.value(&index1), Ok(1_i64));
        assert_eq!(storage.value(&index3), Ok(3_i64));
    }

    #[test]
    fn shrink_to_fit_no_change() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();

        let actual_size = metadata(test_file.file_name()).unwrap().len();

        storage.shrink_to_fit().unwrap();

        assert_eq!(actual_size, metadata(test_file.file_name()).unwrap().len());
        assert_eq!(storage.value(&index1), Ok(1_i64));
        assert_eq!(storage.value(&index2), Ok(2_i64));
        assert_eq!(storage.value(&index3), Ok(3_i64));
    }

    #[test]
    fn shrink_to_fit_uncommitted() {
        let test_file = TestFile::new();

        let expected_size;
        let index1;
        let index2;
        let index3;

        {
            let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
            index1 = storage.insert(&1_i64).unwrap();
            index2 = storage.insert(&2_i64).unwrap();
            index3 = storage.insert(&3_i64).unwrap();
            storage.remove(&index2).unwrap();

            expected_size = metadata(test_file.file_name()).unwrap().len();

            storage.transaction();
            storage.shrink_to_fit().unwrap();
        }

        let actual_size = metadata(test_file.file_name()).unwrap().len();
        assert_eq!(actual_size, expected_size);

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(storage.value(&index1), Ok(1_i64));
        assert_eq!(
            storage.value::<i64>(&index2),
            Err(DbError::from(format!(
                "index '{}' not found",
                index2.value()
            )))
        );
        assert_eq!(storage.value(&index3), Ok(3_i64));
    }

    #[test]
    fn transaction_commit() {
        let test_file = TestFile::from("file_storage-transaction_commit.agdb");
        let index;

        {
            let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            storage.commit().unwrap();
            assert_eq!(storage.value::<i64>(&index), Ok(1_i64));
        }

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(storage.value::<i64>(&index), Ok(1_i64));
    }

    #[test]
    fn transaction_commit_no_transaction() {
        let test_file = TestFile::from("file_storage-transaction_commit_no_transaction.agdb");
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(storage.commit(), Ok(()));
    }

    #[test]
    fn transaction_unfinished() {
        let test_file = TestFile::new();
        let index;

        {
            let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(&index), Ok(1_i64));
        }

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(
            storage.value::<i64>(&index),
            Err(DbError::from(format!(
                "index '{}' not found",
                index.value()
            )))
        );
    }

    #[test]
    fn transaction_nested_unfinished() {
        let test_file = TestFile::new();
        let index;

        {
            let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
            storage.transaction();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(&index), Ok(1_i64));
            storage.commit().unwrap();
        }

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(
            storage.value::<i64>(&index),
            Err(DbError::from(format!(
                "index '{}' not found",
                index.value()
            )))
        );
    }

    #[test]
    fn value() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(storage.value::<i64>(&index), Ok(10_i64));
    }

    #[test]
    fn value_at() {
        let test_file = TestFile::new();

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        let data = vec![1_i64, 2_i64, 3_i64];

        let index = storage.insert(&data).unwrap();
        let offset = u64::fixed_size() + i64::fixed_size();

        assert_eq!(storage.value_at::<i64>(&index, offset), Ok(2_i64));
    }

    #[test]
    fn value_at_dynamic_size() {
        let test_file = TestFile::new();

        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();
        let data = vec![2_i64, 1_i64, 2_i64];

        let index = storage.insert(&data).unwrap();
        let offset = u64::fixed_size();

        assert_eq!(
            storage.value_at::<Vec<i64>>(&index, offset),
            Ok(vec![1_i64, 2_i64])
        );
    }

    #[test]
    fn value_at_of_missing_index() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.value_at::<i64>(&StorageIndex::from(1_i64), 8),
            Err(DbError::from("index '1' not found"))
        );
    }

    #[test]
    fn value_at_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        let data = vec![1_i64, 2_i64];
        let index = storage.insert(&data).unwrap();
        let offset = (u64::fixed_size() + i64::fixed_size() * 2) as u64;

        assert_eq!(
            storage.value_at::<i64>(&index, offset),
            Err(DbError::from("deserialization error: value out of bounds"))
        );
    }

    #[test]
    fn value_at_offset_overflow() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        let data = vec![1_i64, 2_i64];
        let index = storage.insert(&data).unwrap();
        let offset = (u64::fixed_size() + i64::fixed_size() * 3) as u64;

        assert_eq!(
            storage.value_at::<i64>(&index, offset),
            Err(DbError::from("deserialization error: offset out of bounds"))
        );
    }

    #[test]
    fn value_of_missing_index() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.value::<i64>(&StorageIndex::from(1_i64)),
            Err(DbError::from("index '1' not found"))
        );
    }

    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(&index),
            Err(DbError::from("i64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn value_size() {
        let test_file = TestFile::new();
        let mut storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::fixed_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));
    }

    #[test]
    fn value_size_of_missing_index() {
        let test_file = TestFile::new();
        let storage = StorageFile::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.value_size(&StorageIndex::from(1_i64)),
            Err(DbError::from("index '1' not found"))
        );
    }
}
