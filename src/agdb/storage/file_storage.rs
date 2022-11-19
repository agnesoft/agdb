use super::file_record::FileRecord;
use super::file_records::FileRecords;
use super::write_ahead_log::WriteAheadLog;
use super::write_ahead_log::WriteAheadLogRecord;
use super::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeFixedSized;
use crate::DbError;
use crate::DbIndex;
use std::cell::RefCell;
use std::cmp::max;
use std::cmp::min;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

pub struct FileStorage {
    file: RefCell<File>,
    file_records: FileRecords,
    transactions: u64,
    wal: WriteAheadLog,
}

impl FileStorage {
    #[allow(dead_code)]
    pub fn new(filename: &String) -> Result<Self, DbError> {
        let mut data = FileStorage {
            file: RefCell::new(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(filename)?,
            ),
            file_records: FileRecords::new(),
            transactions: 0,
            wal: WriteAheadLog::new(filename)?,
        };

        data.apply_wal()?;
        data.read_records()?;

        Ok(data)
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

    fn begin_transaction(&mut self) {
        self.transactions += 1;
    }

    fn end_transaction(&mut self) -> Result<(), DbError> {
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
        self.write(
            record.pos,
            &DbIndex::from_values(record.index, new_size).serialize(),
        )?;

        self.append(&vec![0_u8; (new_size - record.size) as usize])?;

        self.set_size(record.index, new_size);
        record.size = new_size;

        Ok(())
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

    fn is_at_end(&self, record: &FileRecord) -> Result<bool, DbError> {
        Ok(self.len()? == record.end())
    }

    fn move_to_end(&mut self, record: &mut FileRecord, new_size: u64) -> Result<(), DbError> {
        let mut bytes = self.read_value(record)?;
        bytes.resize(new_size as usize, 0_u8);

        let len = self.len()?;
        self.update_record(record, len, new_size)?;

        self.append(&DbIndex::from_values(record.index, record.size).serialize())?;
        self.append(&bytes)
    }

    fn new_record(&mut self, pos: u64, value_len: u64) -> FileRecord {
        self.file_records.new_record(pos, value_len)
    }

    fn read_exact(&self, pos: u64, value_len: u64) -> Result<Vec<u8>, DbError> {
        self.file.borrow_mut().seek(SeekFrom::Start(pos))?;

        let mut buffer = vec![0_u8; value_len as usize];
        self.file.borrow_mut().read_exact(&mut buffer)?;

        Ok(buffer)
    }

    fn read_record(&mut self) -> Result<FileRecord, DbError> {
        let pos = self.file.borrow_mut().seek(SeekFrom::Current(0))?;
        let bytes = self.read_exact(pos, DbIndex::serialized_size())?;
        let index = DbIndex::deserialize(&bytes)?;
        self.file.borrow_mut().seek(SeekFrom::Start(
            pos + DbIndex::serialized_size() + index.meta(),
        ))?;

        Ok(FileRecord {
            index: index.value(),
            pos,
            size: index.meta(),
        })
    }

    fn read_records(&mut self) -> Result<(), DbError> {
        let mut records: Vec<FileRecord> = vec![FileRecord::default()];
        let len = self.len()?;
        self.file.borrow_mut().seek(SeekFrom::Start(0))?;

        while self.file.borrow_mut().seek(SeekFrom::Current(0))? < len {
            let record = self.read_record()?;
            let index = record.index as usize;

            if records.len() <= index {
                records.resize(index + 1, FileRecord::default());
            }

            records[index] = record;
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
        Ok(self.file.borrow_mut().set_len(len)?)
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
            self.write(
                current_pos,
                &DbIndex::from_values(record.index, record.size).serialize(),
            )?;
            self.write(current_pos + DbIndex::serialized_size(), &bytes)?;
        }

        Ok(current_pos + DbIndex::serialized_size() + record.size)
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

        Ok(())
    }

    fn validate_read_size(offset: u64, read_size: u64, value_size: u64) -> Result<(), DbError> {
        if offset > value_size {
            return Err(DbError::from(format!(
                "FileStorage error: offset ({}) out of bounds ({})",
                offset, value_size
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
        self.file.borrow_mut().seek(SeekFrom::Start(pos))?;

        Ok(self.file.borrow_mut().write_all(bytes)?)
    }
}

impl Storage for FileStorage {
    fn commit(&mut self) -> Result<(), DbError> {
        self.end_transaction()
    }

    fn insert<T: Serialize>(&mut self, value: &T) -> Result<DbIndex, DbError> {
        self.insert_bytes(&value.serialize())
    }

    fn insert_at<T: Serialize>(
        &mut self,
        index: &DbIndex,
        offset: u64,
        value: &T,
    ) -> Result<u64, DbError> {
        self.insert_bytes_at(index, offset, &value.serialize())
    }

    fn insert_bytes(&mut self, bytes: &[u8]) -> Result<DbIndex, DbError> {
        let len = self.len()?;
        let record = self.new_record(len, bytes.len() as u64);
        let index = DbIndex::from_values(record.index, record.size);

        self.transaction();
        self.append(&index.serialize())?;
        self.append(bytes)?;
        self.commit()?;

        Ok(index)
    }

    fn insert_bytes_at(
        &mut self,
        index: &DbIndex,
        offset: u64,
        bytes: &[u8],
    ) -> Result<u64, DbError> {
        let mut record = self.record(index.value())?;

        self.transaction();
        self.ensure_size(&mut record, offset, bytes.len() as u64)?;
        let pos = record.value_start() + offset;
        self.write(pos, bytes)?;
        self.commit()?;

        Ok(record.size)
    }

    fn len(&self) -> Result<u64, DbError> {
        Ok(self.file.borrow_mut().seek(SeekFrom::End(0))?)
    }

    fn move_at(
        &mut self,
        index: &DbIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<u64, DbError> {
        let bytes = self.value_as_bytes_at_size(index, offset_from, size)?;

        self.transaction();
        let value_len = self.insert_bytes_at(index, offset_to, &bytes)?;
        let record = self.record(index.value())?;
        self.erase_bytes(record.value_start(), offset_from, offset_to, size)?;
        self.commit()?;

        Ok(value_len)
    }

    fn remove(&mut self, index: &DbIndex) -> Result<(), DbError> {
        let record = self.record(index.value())?;
        self.remove_index(index.value());

        self.transaction();
        self.invalidate_record(record.pos)?;
        self.commit()
    }

    fn replace<T: Serialize>(&mut self, index: &DbIndex, value: &T) -> Result<u64, DbError> {
        self.replace_with_bytes(index, &value.serialize())
    }

    fn replace_with_bytes(&mut self, index: &DbIndex, bytes: &[u8]) -> Result<u64, DbError> {
        self.transaction();
        self.insert_bytes_at(index, 0, bytes)?;
        let len = self.resize_value(index, bytes.len() as u64)?;
        self.commit()?;

        Ok(len)
    }

    #[allow(clippy::comparison_chain)]
    fn resize_value(&mut self, index: &DbIndex, new_size: u64) -> Result<u64, DbError> {
        let mut record = self.record(index.value())?;

        self.transaction();

        if new_size > record.size {
            self.enlarge_value(&mut record, new_size)?;
        } else if new_size < record.size {
            self.shrink_value(&mut record, new_size)?;
        }

        self.commit()?;

        Ok(record.size)
    }

    fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.transaction();
        let records = self.records();
        let size = self.shrink_records(records)?;
        self.truncate(size)?;

        self.commit()
    }

    fn transaction(&mut self) {
        self.begin_transaction();
    }

    fn value<T: Serialize>(&self, index: &DbIndex) -> Result<T, DbError> {
        T::deserialize(&self.value_as_bytes(index)?)
    }

    fn value_as_bytes(&self, index: &DbIndex) -> Result<Vec<u8>, DbError> {
        self.value_as_bytes_at(index, 0)
    }

    fn value_as_bytes_at(&self, index: &DbIndex, offset: u64) -> Result<Vec<u8>, DbError> {
        let size = self.value_size(index)?;
        self.value_as_bytes_at_size(index, offset, size - min(size, offset))
    }

    fn value_as_bytes_at_size(
        &self,
        index: &DbIndex,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, DbError> {
        let record = self.record(index.value())?;
        Self::validate_read_size(offset, size, record.size)?;
        let pos = record.value_start() + offset;

        self.read_exact(pos, size)
    }

    fn value_at<T: Serialize>(&self, index: &DbIndex, offset: u64) -> Result<T, DbError> {
        T::deserialize(&self.value_as_bytes_at(index, offset)?)
    }

    fn value_size(&self, index: &DbIndex) -> Result<u64, DbError> {
        Ok(self.record(index.value())?.size)
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
    use std::fs::metadata;

    use super::*;
    use crate::test_utilities::test_file::TestFile;
    use crate::utilities::serialize::SerializeDynamicSized;
    use crate::utilities::serialize::SerializeFixedSized;

    #[test]
    fn bad_file() {
        assert!(FileStorage::new(&"/a/".to_string()).is_err());
    }

    #[test]
    fn index_reuse() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let _index1 = storage.insert(&"Hello, World!".to_string()).unwrap();
        let index2 = storage.insert(&10_i64).unwrap();
        let _index3 = storage.insert(&vec![1_u64, 2_u64, 3_u64]).unwrap();

        storage.remove(&index2).unwrap();

        let index4 = storage
            .insert(&vec!["Hello".to_string(), "World".to_string()])
            .unwrap();

        assert_eq!(index2.value(), index4.value());
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

            storage.remove(&index2).unwrap();
        }

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index4 = storage
            .insert(&vec!["Hello".to_string(), "World".to_string()])
            .unwrap();

        assert_eq!(index2.value(), index4.value());
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let value1 = "Hello, World!".to_string();
        let index1 = storage.insert(&value1).unwrap();
        assert!(index1.is_valid());
        assert_eq!(
            storage.value_size(&index1),
            Ok(value1.serialized_size() as u64)
        );
        assert_eq!(storage.value_size(&index1), Ok(index1.meta()));
        assert_eq!(storage.value(&index1), Ok(value1));

        let value2 = 10_i64;
        let index2 = storage.insert(&value2).unwrap();
        assert!(index2.is_valid());
        assert_eq!(
            storage.value_size(&index2),
            Ok(i64::serialized_size() as u64)
        );
        assert_eq!(storage.value_size(&index2), Ok(index2.meta()));
        assert_eq!(storage.value(&index2), Ok(value2));

        let value3 = vec![1_u64, 2_u64, 3_u64];
        let index3 = storage.insert(&value3).unwrap();
        assert!(index3.is_valid());
        assert_eq!(
            storage.value_size(&index3),
            Ok(value3.serialized_size() as u64)
        );
        assert_eq!(storage.value_size(&index3), Ok(index3.meta()));
        assert_eq!(storage.value(&index3), Ok(value3));

        let value4 = vec!["Hello".to_string(), "World".to_string()];
        let index4 = storage.insert(&value4).unwrap();
        assert!(index4.is_valid());
        assert_eq!(
            storage.value_size(&index4),
            Ok(value4.serialized_size() as u64)
        );
        assert_eq!(storage.value_size(&index4), Ok(index4.meta()));
        assert_eq!(storage.value(&index4), Ok(value4));
    }

    #[test]
    fn insert_at() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let size = storage.value_size(&index).unwrap();
        let offset = (u64::serialized_size() + i64::serialized_size()) as u64;

        assert_eq!(storage.insert_at(&index, offset, &10_i64).unwrap(), size);
        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 10_i64, 3_i64]
        );
    }

    #[test]
    fn insert_at_value_end() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = u64::serialized_size() + i64::serialized_size() * 3;
        assert_eq!(storage.insert_at(&index, 0, &4_u64).unwrap(), 32);
        assert_eq!(storage.insert_at(&index, offset, &10_i64).unwrap(), 40);

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_value_end_multiple_values() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        storage.insert(&"Hello, World!".to_string()).unwrap();
        let offset = u64::serialized_size() + i64::serialized_size() * 3;
        assert_eq!(storage.insert_at(&index, 0, &4_u64).unwrap(), 32);
        assert_eq!(storage.insert_at(&index, offset, &10_i64).unwrap(), 40);

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_beyond_end() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset = u64::serialized_size() + i64::serialized_size() * 4;
        assert_eq!(storage.insert_at(&index, 0, &5_u64).unwrap(), 32);
        assert_eq!(storage.insert_at(&index, offset, &10_i64).unwrap(), 48);

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 0_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_beyond_end_multiple_values() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        storage.insert(&"Hello, World!".to_string()).unwrap();
        let offset = u64::serialized_size() + i64::serialized_size() * 4;
        assert_eq!(storage.insert_at(&index, 0, &5_u64).unwrap(), 32);
        assert_eq!(storage.insert_at(&index, offset, &10_i64).unwrap(), 48);

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64, 0_i64, 10_i64]
        );
    }

    #[test]
    fn insert_at_missing_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.insert_at(&DbIndex::from(1_u64), 8, &1_i64),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn move_at_left() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let value_size = storage.value_size(&index).unwrap();
        let offset_from = u64::serialized_size() + i64::serialized_size() * 2;
        let offset_to = u64::serialized_size() + i64::serialized_size();
        let size = i64::serialized_size();

        assert_eq!(
            storage
                .move_at(&index, offset_from, offset_to, size)
                .unwrap(),
            value_size
        );

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 3_i64, 0_i64]
        )
    }

    #[test]
    fn move_at_left_overlapping() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let value_size = storage.value_size(&index).unwrap();
        let offset_from = u64::serialized_size() + i64::serialized_size();
        let offset_to = u64::serialized_size();
        let size = u64::serialized_size() * 2;

        assert_eq!(
            storage
                .move_at(&index, offset_from, offset_to, size)
                .unwrap(),
            value_size
        );

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![2_i64, 3_i64, 0_i64]
        )
    }

    #[test]
    fn move_at_right() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let value_size = storage.value_size(&index).unwrap();
        let offset_from = u64::serialized_size() + i64::serialized_size();
        let offset_to = u64::serialized_size() + i64::serialized_size() * 2;
        let size = u64::serialized_size();

        assert_eq!(
            storage
                .move_at(&index, offset_from, offset_to, size)
                .unwrap(),
            value_size
        );

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 0_i64, 2_i64]
        )
    }

    #[test]
    fn move_at_right_overlapping() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let value_size = storage.value_size(&index).unwrap();
        let offset_from = u64::serialized_size();
        let offset_to = u64::serialized_size() + i64::serialized_size();
        let size = u64::serialized_size() * 2;

        assert_eq!(
            storage
                .move_at(&index, offset_from, offset_to, size)
                .unwrap(),
            value_size
        );

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![0_i64, 1_i64, 2_i64]
        )
    }

    #[test]
    fn move_at_beyond_end() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();
        let offset_from = u64::serialized_size() + i64::serialized_size();
        let offset_to = u64::serialized_size() + i64::serialized_size() * 4;
        let size = u64::serialized_size();

        assert_eq!(
            storage
                .move_at(&index, offset_from, offset_to, size)
                .unwrap(),
            offset_to + size
        );

        storage.insert_at(&index, 0, &5_u64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 0_i64, 3_i64, 0_i64, 2_i64]
        )
    }

    #[test]
    fn move_at_size_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&vec![1_i64, 2_i64, 3_i64]).unwrap();

        assert_eq!(
            storage.move_at(&index, 8, 16, 1000),
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
        let value_size = storage.value_size(&index).unwrap();
        let offset_from = u64::serialized_size();
        let offset_to = u64::serialized_size();
        let size = u64::serialized_size();

        assert_eq!(
            storage
                .move_at(&index, offset_from, offset_to, size)
                .unwrap(),
            value_size
        );

        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64]
        )
    }

    #[test]
    fn move_at_zero_size() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let value = vec![1_i64, 2_i64, 3_i64];
        let index = storage.insert(&value).unwrap();

        assert_eq!(
            storage.move_at(&index, 0, 1, 0).unwrap(),
            value.serialized_size()
        );
        assert_eq!(
            storage.value::<Vec<i64>>(&index).unwrap(),
            vec![1_i64, 2_i64, 3_i64]
        );
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();

        assert_eq!(storage.value::<i64>(&index).unwrap(), 1_i64);

        storage.remove(&index).unwrap();

        assert_eq!(
            storage.value::<i64>(&index),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn remove_missing_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.remove(&DbIndex::from(1_u64)),
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

        assert_eq!(storage.replace(&index, &value).unwrap(), expected_size);
        assert_eq!(storage.value_size(&index).unwrap(), expected_size);
    }

    #[test]
    fn replace_missing_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.replace(&DbIndex::from(1_u64), &10_i64),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn replace_same_size() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let size = storage.value_size(&index).unwrap();

        assert_eq!(storage.replace(&index, &10_i64).unwrap(), size);
        assert_eq!(storage.value_size(&index).unwrap(), size);
    }

    #[test]
    fn replace_smaller() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&"Hello, World!".to_string()).unwrap();
        let value = 1_i64;
        let expected_size = i64::serialized_size();

        assert_eq!(storage.replace(&index, &value).unwrap(), expected_size);
        assert_eq!(storage.value_size(&index).unwrap(), expected_size);
    }

    #[test]
    fn resize_at_end_does_not_move() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&1_i64).unwrap();
        let size = storage.len().unwrap();
        let value_size = storage.value_size(&index).unwrap();

        assert_eq!(
            storage.resize_value(&index, value_size + 8).unwrap(),
            value_size + 8
        );
        assert_eq!(storage.len(), Ok(size + 8));
    }

    #[test]
    fn resize_value_greater() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));
        assert_eq!(
            storage.resize_value(&index, expected_size * 2),
            Ok(expected_size * 2)
        );
        assert_eq!(storage.value_size(&index), Ok(expected_size * 2));
    }

    #[test]
    fn resize_value_missing_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.resize_value(&DbIndex::from(1_u64), 1),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn resize_value_same() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));
        assert_eq!(
            storage.resize_value(&index, expected_size).unwrap(),
            expected_size
        );
        assert_eq!(storage.value_size(&index), Ok(expected_size));
    }

    #[test]
    fn resize_value_smaller() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));
        assert_eq!(
            storage.resize_value(&index, expected_size / 2).unwrap(),
            expected_size / 2
        );
        assert_eq!(storage.value_size(&index), Ok(expected_size / 2));
    }

    #[test]
    fn resize_value_zero() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();
        let expected_size = i64::serialized_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));
        assert_eq!(storage.resize_value(&index, 0), Ok(0));
        assert_eq!(storage.value_size(&index), Ok(0));
    }

    #[test]
    fn resize_value_resizes_file() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&3_i64).unwrap();
        let len = storage.len().unwrap();
        let size = u64::serialized_size() + i64::serialized_size() * 3;
        let expected_len = len + i64::serialized_size() * 3;

        assert_eq!(storage.resize_value(&index, size).unwrap(), size);
        assert_eq!(storage.value::<Vec<i64>>(&index).unwrap(), vec![0_i64; 3]);
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
            storage.resize_value(&index, 1).unwrap();
            storage.remove(&index).unwrap();
        }

        let storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value::<i64>(&index),
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

        let storage = FileStorage::new(test_file.file_name()).unwrap();

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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
            storage.remove(&index2).unwrap();
        }

        let storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(&index1), Ok(value1));
        assert_eq!(
            storage.value::<u64>(&DbIndex::default()),
            Err(DbError::from("FileStorage error: index (0) not found"))
        );
        assert_eq!(
            storage.value::<u64>(&index2),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            index1 = storage.insert(&value1).unwrap();
            index2 = storage.insert(&value2).unwrap();
            index3 = storage.insert(&value3).unwrap();
            storage.remove(&index1).unwrap();
            storage.remove(&index2).unwrap();
            storage.remove(&index3).unwrap();
        }

        let storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value::<u64>(&DbIndex::default()),
            Err(DbError::from("FileStorage error: index (0) not found"))
        );
        assert_eq!(
            storage.value::<Vec<i64>>(&index1),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index1.value()
            )))
        );
        assert_eq!(
            storage.value::<u64>(&index2),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index2.value()
            )))
        );
        assert_eq!(
            storage.value::<Vec<i64>>(&index3),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index3.value()
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
        wal.insert(DbIndex::serialized_size(), 2_u64.serialize())
            .unwrap();

        let storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(storage.value::<Vec<i64>>(&index1), Ok(vec![1_i64, 2_i64]));
        assert_eq!(storage.value::<u64>(&index2), Ok(value2));
        assert_eq!(storage.value::<Vec<i64>>(&index3), Ok(value3));
    }

    #[test]
    fn shrink_to_fit() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index1 = storage.insert(&1_i64).unwrap();
        let index2 = storage.insert(&2_i64).unwrap();
        let index3 = storage.insert(&3_i64).unwrap();
        storage.remove(&index2).unwrap();
        storage.shrink_to_fit().unwrap();

        let actual_size = metadata(test_file.file_name()).unwrap().len();
        let expected_size = (u64::serialized_size() * 2) * 2 + i64::serialized_size() * 2;

        assert_eq!(actual_size, expected_size);
        assert_eq!(storage.value(&index1), Ok(1_i64));
        assert_eq!(storage.value(&index3), Ok(3_i64));
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
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
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

        let storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(storage.value(&index1), Ok(1_i64));
        assert_eq!(
            storage.value::<i64>(&index2),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index2.value()
            )))
        );
        assert_eq!(storage.value(&index3), Ok(3_i64));
    }

    #[test]
    fn transaction_commit() {
        let test_file = TestFile::new();
        let index;

        {
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            storage.commit().unwrap();
            assert_eq!(storage.value::<i64>(&index), Ok(1_i64));
        }

        let storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(storage.value::<i64>(&index), Ok(1_i64));
    }

    #[test]
    fn transaction_commit_no_transaction() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(storage.commit(), Ok(()));
    }

    #[test]
    fn transaction_unfinished() {
        let test_file = TestFile::new();
        let index;

        {
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(&index), Ok(1_i64));
        }

        let storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(
            storage.value::<i64>(&index),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index.value()
            )))
        );
    }

    #[test]
    fn transaction_nested_unfinished() {
        let test_file = TestFile::new();
        let index;

        {
            let mut storage = FileStorage::new(test_file.file_name()).unwrap();
            storage.transaction();
            storage.transaction();
            index = storage.insert(&1_i64).unwrap();
            assert_eq!(storage.value::<i64>(&index), Ok(1_i64));
            storage.commit().unwrap();
        }

        let storage = FileStorage::new(test_file.file_name()).unwrap();
        assert_eq!(
            storage.value::<i64>(&index),
            Err(DbError::from(format!(
                "FileStorage error: index ({}) not found",
                index.value()
            )))
        );
    }

    #[test]
    fn value() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(storage.value::<i64>(&index), Ok(10_i64));
    }

    #[test]
    fn value_at() {
        let test_file = TestFile::new();

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let data = vec![1_i64, 2_i64, 3_i64];

        let index = storage.insert(&data).unwrap();
        let offset = u64::serialized_size() + i64::serialized_size();

        assert_eq!(storage.value_at::<i64>(&index, offset), Ok(2_i64));
    }

    #[test]
    fn value_at_dynamic_size() {
        let test_file = TestFile::new();

        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let data = vec![2_i64, 1_i64, 2_i64];

        let index = storage.insert(&data).unwrap();
        let offset = u64::serialized_size();

        assert_eq!(
            storage.value_at::<Vec<i64>>(&index, offset),
            Ok(vec![1_i64, 2_i64])
        );
    }

    #[test]
    fn value_at_of_missing_index() {
        let test_file = TestFile::new();
        let storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value_at::<i64>(&DbIndex::from(1_u64), 8),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn value_at_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let data = vec![1_i64, 2_i64];
        let index = storage.insert(&data).unwrap();
        let offset = (u64::serialized_size() + i64::serialized_size() * 2) as u64;

        assert_eq!(
            storage.value_at::<i64>(&index, offset),
            Err(DbError::from("i64 deserialization error: out of bounds"))
        );
    }

    #[test]
    fn value_at_offset_overflow() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let data = vec![1_i64, 2_i64];
        let index = storage.insert(&data).unwrap();
        let offset = (u64::serialized_size() + i64::serialized_size() * 3) as u64;

        assert_eq!(
            storage.value_at::<i64>(&index, offset),
            Err(DbError::from(
                "FileStorage error: offset (32) out of bounds (24)"
            ))
        );
    }

    #[test]
    fn value_of_missing_index() {
        let test_file = TestFile::new();
        let storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value::<i64>(&DbIndex::from(1_u64)),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }

    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(&index),
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
        let expected_size = i64::serialized_size();

        assert_eq!(storage.value_size(&index), Ok(expected_size));
    }

    #[test]
    fn value_size_of_missing_index() {
        let test_file = TestFile::new();
        let storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            storage.value_size(&DbIndex::from(1_u64)),
            Err(DbError::from("FileStorage error: index (1) not found"))
        );
    }
}
