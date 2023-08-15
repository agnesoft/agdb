use super::file_records::FileRecord;
use super::file_records::FileRecords;
use super::write_ahead_log::WriteAheadLog;
use super::write_ahead_log::WriteAheadLogRecord;
use super::Storage;
use super::StorageIndex;
use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use std::cmp::max;
use std::cmp::min;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

pub struct FileStorage {
    file: File,
    filename: String,
    file_records: FileRecords,
    transactions: u64,
    wal: WriteAheadLog,
}

impl FileStorage {
    pub fn new(filename: &str) -> Result<Self, DbError> {
        let mut data = FileStorage {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(filename)?,
            filename: filename.to_string(),
            file_records: FileRecords::new(),
            transactions: 0,
            wal: WriteAheadLog::new(filename)?,
        };

        data.apply_wal()?;
        data.read_records()?;

        Ok(data)
    }

    pub fn backup(&mut self, filename: &str) -> Result<(), DbError> {
        self.file.flush()?;
        std::fs::copy(&self.filename, filename)?;
        Ok(())
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), DbError> {
        let len = self.len()?;
        self.write(len, bytes)
    }

    fn apply_wal(&mut self) -> Result<(), DbError> {
        for record in self.wal.records()? {
            self.apply_wal_record(record)?;
        }

        self.wal.clear()
    }

    fn apply_wal_record(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError> {
        if record.value.is_empty() {
            self.set_len(record.pos)
        } else {
            self.write(record.pos, &record.value)
        }
    }

    fn begin_transaction(&mut self) -> u64 {
        self.transactions += 1;
        self.transactions
    }

    fn end_transaction(&mut self, id: u64) -> Result<(), DbError> {
        if self.transactions != id {
            return Err(DbError::from(format!(
                "Cannot end transaction '{id}'. Transaction '{}' in progress.",
                self.transactions
            )));
        }

        if self.transactions != 0 {
            self.transactions -= 1;

            if self.transactions == 0 {
                self.wal.clear()?;
            }
        }

        Ok(())
    }

    fn enlarge_value(&mut self, record: &mut FileRecord, new_size: u64) -> Result<(), DbError> {
        if self.is_at_end(record)? {
            self.enlarge_at_end(record, new_size)
        } else {
            self.move_to_end(record, new_size)
        }
    }

    fn enlarge_at_end(&mut self, record: &mut FileRecord, new_size: u64) -> Result<(), DbError> {
        let old_size = record.size;
        record.size = new_size;
        self.set_size(record.index, new_size);
        self.write(
            record.pos + record.index.serialized_size(),
            &record.size.serialize(),
        )?;
        self.append(&vec![0_u8; (new_size - old_size) as usize])
    }

    fn ensure_size(
        &mut self,
        record: &mut FileRecord,
        offset: u64,
        size: u64,
    ) -> Result<(), DbError> {
        let new_size = offset + size;

        if new_size > record.size {
            self.enlarge_value(record, new_size)?;
        }

        Ok(())
    }

    #[allow(clippy::comparison_chain)]
    fn erase_bytes(
        &mut self,
        pos: u64,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError> {
        if offset_from < offset_to {
            self.write(
                pos + offset_from,
                &vec![0_u8; min(size, offset_to - offset_from) as usize],
            )?;
        } else if offset_from > offset_to {
            let position = max(offset_to + size, offset_from);
            self.write(
                pos + position,
                &vec![0_u8; (offset_from + size - position) as usize],
            )?;
        }

        Ok(())
    }

    fn invalidate_record(&mut self, pos: u64) -> Result<(), DbError> {
        self.write(pos, &0_u64.serialize())
    }

    fn is_at_end(&mut self, record: &FileRecord) -> Result<bool, DbError> {
        Ok(self.len()? == record.end())
    }

    fn move_to_end(&mut self, record: &mut FileRecord, new_size: u64) -> Result<(), DbError> {
        let mut bytes = self.read_value(record)?;
        bytes.resize(new_size as usize, 0_u8);

        let len = self.len()?;
        self.update_record(record, len, new_size)?;
        self.append(&bytes)
    }

    fn new_record(&mut self, pos: u64, value_len: u64) -> FileRecord {
        self.file_records.new_record(pos, value_len)
    }

    fn read_exact(&mut self, pos: u64, value_len: u64) -> Result<Vec<u8>, DbError> {
        self.file.seek(SeekFrom::Start(pos))?;

        let mut buffer = vec![0_u8; value_len as usize];
        self.file.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    fn read_record(&mut self) -> Result<FileRecord, DbError> {
        let pos = self.file.stream_position()?;
        let bytes = self.read_exact(pos, Self::record_serialized_size())?;
        let index = u64::deserialize(&bytes)?;
        let size = u64::deserialize(&bytes[index.serialized_size() as usize..])?;
        self.file
            .seek(SeekFrom::Start(pos + Self::record_serialized_size() + size))?;

        Ok(FileRecord { index, pos, size })
    }

    fn read_records(&mut self) -> Result<(), DbError> {
        let mut records: Vec<FileRecord> = vec![FileRecord::default()];
        let len = self.len()?;
        self.file.seek(SeekFrom::Start(0))?;

        while self.file.stream_position()? < len {
            let record = self.read_record()?;

            if record.index != 0 {
                let index = record.index as usize;

                if records.len() <= index {
                    records.resize(index + 1, FileRecord::default());
                }

                records[index] = record;
            }
        }

        self.file_records.set_records(records);

        Ok(())
    }

    fn read_value(&mut self, record: &FileRecord) -> Result<Vec<u8>, DbError> {
        self.read_exact(record.value_start(), record.size)
    }

    fn record(&self, index: u64) -> Result<FileRecord, DbError> {
        self.file_records.record(index)
    }

    fn record_serialized_size() -> u64 {
        u64::serialized_size_static() * 2
    }

    fn record_wal(&mut self, pos: u64, size: u64) -> Result<(), DbError> {
        if pos >= self.len()? {
            self.wal.insert(pos, vec![])
        } else {
            let bytes = self.read_exact(pos, size)?;

            self.wal.insert(pos, bytes)
        }
    }

    fn records(&self) -> Vec<FileRecord> {
        self.file_records.records()
    }

    fn remove_index(&mut self, index: u64) {
        self.file_records.remove_index(index);
    }

    fn set_len(&mut self, len: u64) -> Result<(), DbError> {
        Ok(self.file.set_len(len)?)
    }

    fn set_pos(&mut self, index: u64, pos: u64) {
        self.file_records.set_pos(index, pos);
    }

    fn set_size(&mut self, index: u64, size: u64) {
        self.file_records.set_size(index, size);
    }

    fn shrink_index(&mut self, record: &FileRecord, current_pos: u64) -> Result<u64, DbError> {
        if record.pos != current_pos {
            let bytes = self.read_value(record)?;
            self.set_pos(record.index, current_pos);
            self.write_record(&FileRecord {
                index: record.index,
                pos: current_pos,
                size: record.size,
            })?;
            self.write(current_pos + Self::record_serialized_size(), &bytes)?;
        }

        Ok(current_pos + Self::record_serialized_size() + record.size)
    }

    fn shrink_records(&mut self, records: Vec<FileRecord>) -> Result<u64, DbError> {
        let mut current_pos = 0_u64;

        for record in records {
            current_pos = self.shrink_index(&record, current_pos)?;
        }

        Ok(current_pos)
    }

    fn shrink_value(&mut self, record: &mut FileRecord, new_size: u64) -> Result<(), DbError> {
        if self.is_at_end(record)? {
            record.size = new_size;
            self.set_size(record.index, new_size);
            self.truncate(record.end())
        } else {
            record.size = new_size;
            self.set_size(record.index, new_size);
            self.move_to_end(record, new_size)
        }
    }

    fn truncate(&mut self, size: u64) -> Result<(), DbError> {
        let current_size = self.len()?;

        if size < current_size {
            self.record_wal(size, current_size - size)?;
            self.set_len(size)?;
        }

        Ok(())
    }

    fn update_record(
        &mut self,
        record: &mut FileRecord,
        new_pos: u64,
        new_size: u64,
    ) -> Result<(), DbError> {
        self.invalidate_record(record.pos)?;
        self.set_pos(record.index, new_pos);
        self.set_size(record.index, new_size);
        record.pos = new_pos;
        record.size = new_size;
        self.write_record(record)
    }

    fn validate_read_size(offset: u64, read_size: u64, value_size: u64) -> Result<(), DbError> {
        if offset > value_size {
            return Err(DbError::from(format!(
                "FileStorage error: offset ({offset}) out of bounds ({value_size})"
            )));
        }

        if (offset + read_size) > value_size {
            return Err(DbError::from(format!(
                "FileStorage error: value size ({}) out of bounds ({})",
                offset + read_size,
                value_size
            )));
        }

        Ok(())
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        self.record_wal(pos, bytes.len() as u64)?;
        self.file.seek(SeekFrom::Start(pos))?;

        Ok(self.file.write_all(bytes)?)
    }

    fn write_record(&mut self, record: &FileRecord) -> Result<(), DbError> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(Self::record_serialized_size() as usize);
        bytes.extend(record.index.serialize());
        bytes.extend(record.size.serialize());

        self.write(record.pos, &bytes)
    }
}

impl Storage for FileStorage {
    fn commit(&mut self, id: u64) -> Result<(), DbError> {
        self.end_transaction(id)
    }

    fn insert<T: Serialize>(&mut self, value: &T) -> Result<StorageIndex, DbError> {
        self.insert_bytes(&value.serialize())
    }

    fn insert_at<T: Serialize>(
        &mut self,
        index: StorageIndex,
        offset: u64,
        value: &T,
    ) -> Result<(), DbError> {
        self.insert_bytes_at(index, offset, &value.serialize())
    }

    fn insert_bytes(&mut self, bytes: &[u8]) -> Result<StorageIndex, DbError> {
        let len = self.len()?;
        let record = self.new_record(len, bytes.len() as u64);

        let id = self.transaction();
        self.write_record(&record)?;
        self.append(bytes)?;
        self.commit(id)?;

        Ok(StorageIndex::from(record.index))
    }

    fn insert_bytes_at(
        &mut self,
        index: StorageIndex,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), DbError> {
        let mut record = self.record(index.0)?;

        let id = self.transaction();
        self.ensure_size(&mut record, offset, bytes.len() as u64)?;
        let pos = record.value_start() + offset;
        self.write(pos, bytes)?;
        self.commit(id)
    }

    fn len(&mut self) -> Result<u64, DbError> {
        Ok(self.file.seek(SeekFrom::End(0))?)
    }

    fn move_at(
        &mut self,
        index: StorageIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError> {
        let bytes = self.value_as_bytes_at_size(index, offset_from, size)?;

        let id = self.transaction();
        self.insert_bytes_at(index, offset_to, &bytes)?;
        let record = self.record(index.0)?;
        self.erase_bytes(record.value_start(), offset_from, offset_to, size)?;
        self.commit(id)
    }

    fn remove(&mut self, index: StorageIndex) -> Result<(), DbError> {
        let record = self.record(index.0)?;

        let id = self.transaction();
        self.remove_index(index.0);
        self.invalidate_record(record.pos)?;
        self.commit(id)
    }

    fn replace<T: Serialize>(&mut self, index: StorageIndex, value: &T) -> Result<(), DbError> {
        self.replace_with_bytes(index, &value.serialize())
    }

    fn replace_with_bytes(&mut self, index: StorageIndex, bytes: &[u8]) -> Result<(), DbError> {
        let id = self.transaction();
        self.insert_bytes_at(index, 0, bytes)?;
        self.resize_value(index, bytes.len() as u64)?;
        self.commit(id)
    }

    #[allow(clippy::comparison_chain)]
    fn resize_value(&mut self, index: StorageIndex, new_size: u64) -> Result<(), DbError> {
        let mut record = self.record(index.0)?;

        let id = self.transaction();

        if new_size > record.size {
            self.enlarge_value(&mut record, new_size)?;
        } else if new_size < record.size {
            self.shrink_value(&mut record, new_size)?;
        }

        self.commit(id)
    }

    fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        let id = self.transaction();
        let records = self.records();
        let size = self.shrink_records(records)?;
        self.truncate(size)?;

        self.commit(id)
    }

    fn transaction(&mut self) -> u64 {
        self.begin_transaction()
    }

    fn value<T: Serialize>(&mut self, index: StorageIndex) -> Result<T, DbError> {
        T::deserialize(&self.value_as_bytes(index)?)
    }

    fn value_as_bytes(&mut self, index: StorageIndex) -> Result<Vec<u8>, DbError> {
        self.value_as_bytes_at(index, 0)
    }

    fn value_as_bytes_at(&mut self, index: StorageIndex, offset: u64) -> Result<Vec<u8>, DbError> {
        let size = self.value_size(index)?;
        self.value_as_bytes_at_size(index, offset, size - min(size, offset))
    }

    fn value_as_bytes_at_size(
        &mut self,
        index: StorageIndex,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, DbError> {
        let record = self.record(index.0)?;
        Self::validate_read_size(offset, size, record.size)?;
        let pos = record.value_start() + offset;

        self.read_exact(pos, size)
    }

    fn value_at<T: Serialize>(&mut self, index: StorageIndex, offset: u64) -> Result<T, DbError> {
        T::deserialize(&self.value_as_bytes_at(index, offset)?)
    }

    fn value_size(&mut self, index: StorageIndex) -> Result<u64, DbError> {
        Ok(self.record(index.0)?.size)
    }
}

impl Drop for FileStorage {
    fn drop(&mut self) {
        if self.apply_wal().is_ok() {
            let _ = self.wal.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;
    use std::fs::metadata;

    #[test]
    fn bad_file() {
        assert!(FileStorage::new("/a/").is_err());
    }

    #[test]
    fn index_reuse() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();

            let _index1 = storage.insert(&"Hello, World!".to_string()).unwrap();
            index2 = storage.insert(&10_i64).unwrap();
            let _index3 = storage.insert(&vec![1_u64, 2_u64, 3_u64]).unwrap();

            storage.remove(index2).unwrap();
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();

            index1 = storage.insert(&"Hello, World!".to_string()).unwrap();
            index2 = storage.insert(&10_i64).unwrap();
            let _index3 = storage.insert(&vec![1_u64, 2_u64, 3_u64]).unwrap();

            storage.remove(index1).unwrap();
            storage.remove(index2).unwrap();
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.insert_at(StorageIndex::from(1_u64), 8, &1_i64),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn move_at_left() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();

        assert_eq!(
            storage.move_at(index, 8, 16, 1000),
            Err(DbError::from(
                "FileStorage error: value size (1008) out of bounds (32)"
            ))
        )
    }

    #[test]
    fn move_at_same_offset() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();

        assert_eq!(storage.value::<i64>(index).unwrap(), 1_i64);

        storage.remove(index).unwrap();

        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn remove_missing_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.remove(StorageIndex::from(1_u64)),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn replace_larger() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let value = "Hello, World!".to_string();
        let expected_size = value.serialized_size();

        storage.replace(index, &value).unwrap();

        assert_eq!(storage.value_size(index).unwrap(), expected_size);
    }

    #[test]
    fn replace_missing_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.replace(StorageIndex::from(1_u64), &10_i64),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn replace_same_size() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let size = storage.value_size(index).unwrap();

        storage.replace(index, &10_i64).unwrap();

        assert_eq!(storage.value_size(index).unwrap(), size);
    }

    #[test]
    fn replace_smaller() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&"Hello, World!".to_string()).unwrap();
        let value = 1_i64;
        let expected_size = i64::serialized_size_static();

        storage.replace(index, &value).unwrap();

        assert_eq!(storage.value_size(index).unwrap(), expected_size);
    }

    #[test]
    fn resize_at_end_does_not_move() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let size = storage.len().unwrap();
        let value_size = storage.value_size(index).unwrap();

        storage.resize_value(index, value_size + 8).unwrap();

        assert_eq!(storage.len(), Ok(size + 8));
    }

    #[test]
    fn resize_value_greater() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, expected_size * 2).unwrap();

        assert_eq!(storage.value_size(index), Ok(expected_size * 2));
    }

    #[test]
    fn resize_value_missing_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.resize_value(StorageIndex::from(1_u64), 1),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn resize_value_same() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, expected_size).unwrap();

        assert_eq!(storage.value_size(index), Ok(expected_size));
    }

    #[test]
    fn resize_value_smaller() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, expected_size / 2).unwrap();

        assert_eq!(storage.value_size(index), Ok(expected_size / 2));
    }

    #[test]
    fn resize_value_zero() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));

        storage.resize_value(index, 0).unwrap();

        assert_eq!(storage.value_size(index), Ok(0));
    }

    #[test]
    fn resize_value_resizes_file() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&3_i64).unwrap();
        let len = storage.len().unwrap();
        let size = u64::serialized_size_static() + i64::serialized_size_static() * 3;
        let expected_len = len + i64::serialized_size_static() * 3;

        storage.resize_value(index, size).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(index).unwrap(), vec![0_i64; 3]);
        assert_eq!(storage.len().unwrap(), expected_len);
    }

    #[test]
    fn resize_value_invalidates_original_position() {
        let test_file = TestFile::new();

        let index;

        {
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            index = storage.insert(&10_i64).unwrap();
            storage.insert(&5_i64).unwrap();
            storage.resize_value(index, 1).unwrap();
            storage.remove(index).unwrap();
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::from("FileStorage error: index (1) not found"))
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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
            storage.remove(index2).unwrap();
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(index1), Ok(value1));
        assert_eq!(
            storage.value::<u64>(StorageIndex::default()),
            Err(DbError::from("FileStorage error: index (0) not found"))
        );
        assert_eq!(
            storage.value::<u64>(index2),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
            storage.remove(index1).unwrap();
            storage.remove(index2).unwrap();
            storage.remove(index3).unwrap();
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value::<u64>(StorageIndex::default()),
            Err(DbError::from("FileStorage error: index (0) not found"))
        );
        assert_eq!(
            storage.value::<Vec<i64>>(index1),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index1.0
            )))
        );
        assert_eq!(
            storage.value::<u64>(index2),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index2.0
            )))
        );
        assert_eq!(
            storage.value::<Vec<i64>>(index3),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
        }

        let mut wal = WriteAheadLog::new(test_file.file_name()).unwrap();
        wal.insert(u64::serialized_size_static() * 2, 2_u64.serialize())
            .unwrap();

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(index1), Ok(vec![1_i64, 2_i64]));
        assert_eq!(storage.value::<u64>(index2), Ok(value2));
        assert_eq!(storage.value::<Vec<i64>>(index3), Ok(value3));
    }

    #[test]
    fn shrink_to_fit() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();
        storage.remove(index2).unwrap();
        storage.shrink_to_fit().unwrap();

        let actual_size = metadata(test_file.file_name()).unwrap().len();
        let expected_size =
            (u64::serialized_size_static() * 2) * 2 + i64::serialized_size_static() * 2;

        assert_eq!(actual_size, expected_size);
        assert_eq!(storage.value(index1), Ok(1_i64));
        assert_eq!(storage.value(index3), Ok(3_i64));
    }

    #[test]
    fn shrink_to_fit_no_change() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();

        let actual_size = metadata(test_file.file_name()).unwrap().len();

        storage.shrink_to_fit().unwrap();

        assert_eq!(actual_size, metadata(test_file.file_name()).unwrap().len());
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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&1_i64).unwrap();
            index2 = storage.insert(&2_i64).unwrap();
            index3 = storage.insert(&3_i64).unwrap();
            storage.remove(index2).unwrap();

            expected_size = metadata(test_file.file_name()).unwrap().len();

            storage.transaction();
            storage.shrink_to_fit().unwrap();
        }

        let actual_size = metadata(test_file.file_name()).unwrap().len();
        assert_eq!(actual_size, expected_size);

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(storage.value(index1), Ok(1_i64));
        assert_eq!(
            storage.value::<i64>(index2),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            let id = storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            storage.commit(id).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(storage.value::<i64>(index), Ok(1_i64));
    }

    #[test]
    fn transaction_commit_no_transaction() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(storage.commit(0), Ok(()));
    }

    #[test]
    fn transaction_unfinished() {
        let test_file = TestFile::new();
        let index;

        {
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index.0
            )))
        );
    }

    #[test]
    fn transaction_nested_unfinished() {
        let test_file = TestFile::new();
        let index;

        {
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            let _ = storage.transaction();
            let id2 = storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(index), Ok(1_i64));
            storage.commit(id2).unwrap();
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(
            storage.value::<i64>(index),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index.0
            )))
        );
    }

    #[test]
    fn transaction_commit_mismatch() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(storage.value::<i64>(index), Ok(10_i64));
    }

    #[test]
    fn value_at() {
        let test_file = TestFile::new();

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let data = vec![1_i64, 2_i64, 3_i64];

        let index = storage.insert(&data).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static();

        assert_eq!(storage.value_at::<i64>(index, offset), Ok(2_i64));
    }

    #[test]
    fn value_at_dynamic_size() {
        let test_file = TestFile::new();

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value_at::<i64>(StorageIndex::from(1_u64), 8),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn value_at_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let data = vec![1_i64, 2_i64];
        let index = storage.insert(&data).unwrap();
        let offset = u64::serialized_size_static() + i64::serialized_size_static() * 3;

        assert_eq!(
            storage.value_at::<i64>(index, offset),
            Err(DbError::from(
                "FileStorage error: offset (32) out of bounds (24)"
            ))
        );
    }

    #[test]
    fn value_of_missing_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value::<i64>(StorageIndex::from(1_u64)),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size_static();

        assert_eq!(storage.value_size(index), Ok(expected_size));
    }

    #[test]
    fn value_size_of_missing_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value_size(StorageIndex::from(1_u64)),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }
}
