use std::io::Read;
use std::io::Seek;
use std::io::Write;

use super::file_record::FileRecord;
use super::file_record_full::FileRecordFull;
use super::file_records::FileRecords;
use super::storage_impl::StorageImpl;
use super::write_ahead_log::WriteAheadLog;
use super::write_ahead_log_record::WriteAheadLogRecord;
use super::Storage;
use crate::db_error::DbError;

#[allow(dead_code)]
pub(crate) struct FileStorage {
    file: std::fs::File,
    filename: String,
    records: FileRecords,
    wal: WriteAheadLog,
    wal_filename: String,
    transactions: u64,
}

impl StorageImpl for FileStorage {
    fn begin_transaction(&mut self) {
        self.transactions += 1;
    }

    fn clear_wal(&mut self) -> Result<(), DbError> {
        self.wal.clear()
    }

    fn create_index(&mut self, position: u64, size: u64) -> i64 {
        self.records.create(position, size)
    }

    fn end_transaction(&mut self) -> bool {
        if self.transactions != 0 {
            self.transactions -= 1;
        }

        self.transactions == 0
    }

    fn indexes_by_position(&self) -> Vec<i64> {
        self.records.indexes_by_position()
    }

    fn insert_wal_record(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError> {
        self.wal.insert(record)
    }

    fn read_exact(&mut self, buffer: &mut Vec<u8>) -> Result<(), DbError> {
        Ok(self.file.read_exact(buffer)?)
    }

    fn record(&self, index: i64) -> Result<FileRecord, DbError> {
        Ok(self
            .records
            .get(index)
            .ok_or_else(|| DbError::Storage(format!("index '{}' not found", index)))?
            .clone())
    }

    fn record_mut(&mut self, index: i64) -> &mut FileRecord {
        self.records
            .get_mut(index)
            .expect("validated by previous call to FileStorage::record()")
    }

    fn remove_index(&mut self, index: i64) {
        self.records.remove(index);
    }

    fn seek(&mut self, position: std::io::SeekFrom) -> Result<u64, DbError> {
        Ok(self.file.seek(position)?)
    }

    fn set_len(&mut self, len: u64) -> Result<(), DbError> {
        Ok(self.file.set_len(len)?)
    }

    fn set_records(&mut self, records: Vec<FileRecordFull>) {
        self.records = FileRecords::from(records);
    }

    fn wal_records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError> {
        self.wal.records()
    }

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), DbError> {
        Ok(self.file.write_all(bytes)?)
    }
}

impl Storage for FileStorage {}

impl Drop for FileStorage {
    fn drop(&mut self) {
        if self.apply_wal().is_ok() {
            let _ignore1 = self.wal.clear();
            let _ignore2 = std::fs::remove_file(&self.wal_filename);
        }
    }
}

impl TryFrom<&str> for FileStorage {
    type Error = DbError;

    fn try_from(filename: &str) -> Result<Self, Self::Error> {
        Self::try_from(filename.to_string())
    }
}

fn wal_filename(filename: &str) -> String {
    let pos;

    if let Some(slash) = filename.rfind('/') {
        pos = slash + 1;
    } else if let Some(backslash) = filename.rfind('\\') {
        pos = backslash + 1
    } else {
        pos = 1;
    }

    let mut copy = filename.to_owned();
    copy.insert(pos, '.');
    copy
}

impl TryFrom<String> for FileStorage {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        let wal_filename = wal_filename(&filename);

        let mut storage = FileStorage {
            file: std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .read(true)
                .open(&filename)?,
            filename,
            records: FileRecords::default(),
            wal: WriteAheadLog::try_from(&wal_filename)?,
            wal_filename,
            transactions: 0,
        };

        storage.apply_wal()?;
        storage.read_records()?;

        Ok(storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn insert() {
        let test_file = TestFile::from("./file_storage-insert.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&10_i64);

        assert_eq!(index, Ok(1));
    }

    #[test]
    fn insert_at() {
        let test_file = TestFile::from("./file_storage-insert_at.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = (std::mem::size_of::<u64>() + std::mem::size_of::<i64>()) as u64;
        storage.insert_at(index, offset, &10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 10_i64, 3_i64]
        );
    }

    #[test]
    fn insert_at_missing_index() {
        let test_file = TestFile::from("./file_storage-insert_at_missing_index.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

        assert_eq!(
            storage.insert_at(1, 8, &1_i64),
            Err(DbError::Storage("index '1' not found".to_string()))
        );
    }

    #[test]
    fn insert_at_value_end() {
        let test_file = TestFile::from("./file_storage-insert_at_value_end.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = (std::mem::size_of::<u64>() + std::mem::size_of::<i64>() * 3) as u64;
        storage.insert_at(index, 0, &4_u64).unwrap();
        storage.insert_at(index, offset, &10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_beyond_end() {
        let test_file = TestFile::from("./file_storage-insert_at_beyond_end.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = (std::mem::size_of::<u64>() + std::mem::size_of::<i64>() * 4) as u64;
        storage.insert_at(index, 0, &5_u64).unwrap();
        storage.insert_at(index, offset, &10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 0_i64, 10_i64]
        );
    }

    #[test]
    fn remove() {
        let test_file = TestFile::from("./file_storage-remove.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        storage.remove(index).unwrap();

        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::Storage("index '1' not found".to_string()))
        );
    }

    #[test]
    fn remove_missing_index() {
        let test_file = TestFile::from("./file_storage-remove_missing_index.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

        assert_eq!(
            storage.remove(1_i64),
            Err(DbError::Storage("index '1' not found".to_string()))
        );
    }

    #[test]
    fn restore_from_open_file() {
        let test_file = TestFile::from("./file_storage-restore_from_open_file.agdb");
        let value1 = vec![1_i64, 2_i64, 3_i64];
        let value2 = 64_u64;
        let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
        let index1;
        let index2;
        let index3;

        {
            let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
        }

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(index1), Ok(value1));
        assert_eq!(storage.value::<u64>(index2), Ok(value2));
        assert_eq!(storage.value::<Vec<i64>>(index3), Ok(value3));
    }

    #[test]
    fn restore_from_open_file_with_removed_index() {
        let test_file =
            TestFile::from("./file_storage-restore_from_open_file_with_removed_index.agdb");
        let value1 = vec![1_i64, 2_i64, 3_i64];
        let value2 = 64_u64;
        let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
        let index1;
        let index2;
        let index3;

        {
            let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
            storage.remove(index2).unwrap();
        }

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(index1), Ok(value1));
        assert_eq!(
            storage.value::<u64>(0),
            Err(DbError::Storage(format!("index '{}' not found", 0)))
        );
        assert_eq!(
            storage.value::<u64>(index2),
            Err(DbError::Storage(format!("index '{}' not found", index2)))
        );
        assert_eq!(storage.value::<Vec<i64>>(index3), Ok(value3));
    }

    #[test]
    fn shrink_to_fit() {
        let test_file = TestFile::from("./file_storage-shrink_to_fit.agdb");

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();
        storage.remove(index2).unwrap();
        storage.shrink_to_fit().unwrap();

        let actual_size = std::fs::metadata(test_file.file_name()).unwrap().len();
        let expected_size = std::mem::size_of::<FileRecord>() * 2 + std::mem::size_of::<i64>() * 2;

        assert_eq!(actual_size, expected_size as u64);
        assert_eq!(storage.value(index1), Ok(1_i64));
        assert_eq!(storage.value(index3), Ok(3_i64));
    }

    #[test]
    fn shrink_to_fit_no_change() {
        let test_file = TestFile::from("./file_storage-shrink_to_fit_no_change.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
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
        let test_file = TestFile::from("./file_storage-shrink_to_fit_uncommitted.agdb");

        let expected_size;
        let index1;
        let index2;
        let index3;

        {
            let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
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

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(storage.value(index1), Ok(1_i64));
        assert_eq!(
            storage.value::<i64>(index2),
            Err(DbError::Storage(format!("index '{}' not found", index2)))
        );
        assert_eq!(storage.value(index3), Ok(3_i64));
    }

    #[test]
    fn transaction_commit() {
        let test_file = TestFile::from(".\\\\file_storage-transaction_commit.agdb");
        let index;

        {
            let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            storage.commit().unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
        }

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(storage.value::<i64>(index), Ok(1_i64));
    }

    #[test]
    fn transaction_commit_no_transaction() {
        let test_file = TestFile::from("file_storage-transaction_commit_no_transaction.agdb");

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(storage.commit(), Ok(()));
    }

    #[test]
    fn transaction_unfinished() {
        let test_file = TestFile::from("./file_storage-transaction_unfinished.agdb");
        let index;

        {
            let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
        }

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::Storage(format!("index '{}' not found", index)))
        );
    }

    #[test]
    fn transaction_nested_unfinished() {
        let test_file = TestFile::from("./file_storage-transaction_nested_unfinished.agdb");
        let index;

        {
            let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
            storage.transaction();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
            storage.commit().unwrap();
        }

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::Storage(format!("index '{}' not found", index)))
        );
    }

    #[test]
    fn value() {
        let test_file = TestFile::from("./file_storage-value.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(storage.value::<i64>(index), Ok(10_i64));
    }

    #[test]
    fn value_at() {
        let test_file = TestFile::from("./file_storage-value_at.agdb");

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        let data = vec![1_i64, 2_i64, 3_i64];

        let index = storage.insert(&data).unwrap();
        let offset = (std::mem::size_of::<u64>() + std::mem::size_of::<i64>()) as u64;

        assert_eq!(storage.value_at::<i64>(index, offset), Ok(2_i64));
    }

    #[test]
    fn value_at_of_missing_index() {
        let test_file = TestFile::from("./file_storage-value_at_of_missing_index.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.value_at::<i64>(1, 8),
            Err(DbError::Storage("index '1' not found".to_string()))
        );
    }

    #[test]
    fn value_at_out_of_bounds() {
        let test_file = TestFile::from("./file_storage-value_at_out_of_bounds.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        let data = vec![1_i64, 2_i64];
        let index = storage.insert(&data).unwrap();
        let offset = (std::mem::size_of::<u64>() + std::mem::size_of::<i64>() * 2) as u64;

        assert_eq!(
            storage.value_at::<i64>(index, offset),
            Err(DbError::Storage(
                "deserialization error: value out of bounds".to_string()
            ))
        );
    }

    #[test]
    fn value_at_offset_overflow() {
        let test_file = TestFile::from("./file_storage-value_at_offset_overflow.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        let data = vec![1_i64, 2_i64];
        let index = storage.insert(&data).unwrap();
        let offset = (std::mem::size_of::<u64>() + std::mem::size_of::<i64>() * 3) as u64;

        assert_eq!(
            storage.value_at::<i64>(index, offset),
            Err(DbError::Storage(
                "deserialization error: offset out of bounds".to_string()
            ))
        );
    }

    #[test]
    fn value_of_missing_index() {
        let test_file = TestFile::from("./file_storage-value_of_missing_index.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.value::<i64>(1),
            Err(DbError::Storage("index '1' not found".to_string()))
        );
    }

    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::from("./file_storage-value_out_of_bounds.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index),
            Err(DbError::Storage(
                "i64 deserialization error: out of bounds".to_string()
            ))
        );
    }

    #[test]
    fn value_size() {
        let test_file = TestFile::from("./file_storage-value_size.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = std::mem::size_of::<i64>() as u64;

        assert_eq!(storage.value_size(index), Ok(expected_size));
    }

    #[test]
    fn value_size_of_missing_index() {
        let test_file = TestFile::from("./file_storage-value_size_of_missing_index.agdb");
        let storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.value_size(1),
            Err(DbError::Storage("index '1' not found".to_string()))
        );
    }
}
