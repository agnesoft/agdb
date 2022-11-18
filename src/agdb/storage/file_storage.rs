use super::file_records::FileRecord;
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

    fn enlarge_value(&mut self, record: &mut FileRecord, new_size: u64) -> Result<u64, DbError> {
        let mut bytes = self.read_value(record)?;
        bytes.resize(new_size as usize, 0_u8);

        let len = self.len()?;
        self.update_record(record, len, new_size)?;

        self.append(&DbIndex::from_values(record.index, record.size).serialize())?;
        self.append(&bytes)?;

        Ok(new_size)
    }

    fn ensure_size(
        &mut self,
        record: &mut FileRecord,
        offset: u64,
        size: u64,
    ) -> Result<u64, DbError> {
        let new_size = offset + size;

        if new_size == record.size {
            return Ok(new_size);
        }

        if new_size > record.size {
            self.enlarge_value(record, new_size)?;
        }

        Ok(record.pos)
    }

    fn erase_bytes(
        &mut self,
        record: &FileRecord,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError> {
        if offset_from < offset_to {
            self.write(
                record.pos + offset_from,
                &vec![0_u8; min(size, offset_to - offset_from) as usize],
            )
        } else {
            let position = max(offset_to + size, offset_from);
            self.write(
                record.pos + position,
                &vec![0_u8; (offset_from + size - position) as usize],
            )
        }
    }

    fn invalidate_record(&mut self, pos: u64) -> Result<(), DbError> {
        self.write(pos, &0_u64.serialize())
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
        let bytes = self.read_exact(pos, DbIndex::fixed_serialized_size())?;
        let index = DbIndex::deserialize(&bytes)?;
        self.file.borrow_mut().seek(SeekFrom::Start(
            pos + DbIndex::fixed_serialized_size() + index.meta(),
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
            records.push(self.read_record()?);
        }

        self.file_records.set_records(records);

        Ok(())
    }

    fn read_value(&mut self, record: &FileRecord) -> Result<Vec<u8>, DbError> {
        self.read_exact(record.pos, record.size)
    }

    fn record(&self, index: u64) -> Result<FileRecord, DbError> {
        self.file_records.record(index)
    }

    fn record_wal(&mut self, pos: u64, size: u64) -> Result<(), DbError> {
        if pos == self.len()? {
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

    fn shrink_index(&mut self, record: &FileRecord, mut current_pos: u64) -> Result<u64, DbError> {
        if record.pos != current_pos {
            let bytes = self.read_value(record)?;
            self.set_pos(record.index, current_pos);
            self.write(
                current_pos,
                &DbIndex::from_values(record.index, record.size).serialize(),
            )?;
            self.write(current_pos + DbIndex::fixed_serialized_size(), &bytes)?;
        }

        current_pos += DbIndex::fixed_serialized_size() + record.size;

        Ok(current_pos)
    }

    fn shrink_records(&mut self, records: Vec<FileRecord>) -> Result<u64, DbError> {
        let mut current_pos = 0_u64;

        for record in records {
            current_pos = self.shrink_index(&record, current_pos)?;
        }

        Ok(current_pos)
    }

    fn shrink_value(&mut self, record: &mut FileRecord, new_size: u64) -> Result<u64, DbError> {
        let bytes = self.read_value(record)?;

        let len = self.len()?;
        self.update_record(record, len, new_size)?;

        self.append(&DbIndex::from_values(record.index, record.size).serialize())?;
        self.append(&bytes)?;

        Ok(new_size)
    }

    fn truncate(&mut self, size: u64) -> Result<(), DbError> {
        let current_size = self.file.borrow_mut().seek(SeekFrom::End(0))?;

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
                "FileStorage read error: offset ({}) out of bounds ({})",
                offset, value_size
            )));
        }

        if (offset + read_size) > value_size {
            return Err(DbError::from(format!(
                "FileStorage read error: value ({}) out of bounds ({})",
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
        let pos = self.ensure_size(&mut record, offset, bytes.len() as u64)?;
        self.write(pos + offset, bytes)?;
        self.commit()?;

        Ok(record.size)
    }

    fn len(&self) -> Result<u64, DbError> {
        Ok(self.file.borrow_mut().seek(SeekFrom::End(0))?)
    }

    fn move_at<T: Serialize>(
        &mut self,
        index: &DbIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<u64, DbError> {
        let bytes = self.value_as_bytes_at_size(index, offset_from, size)?;
        let record = self.record(index.value())?;

        self.transaction();
        let value_len = self.insert_bytes_at(index, offset_to, &bytes)?;
        self.erase_bytes(&record, offset_from, offset_to, size)?;
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
        self.value_as_bytes_at_size(index, offset, self.value_size(index)?)
    }

    fn value_as_bytes_at_size(
        &self,
        index: &DbIndex,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, DbError> {
        let record = self.record(index.value())?;
        Self::validate_read_size(offset, size, record.size)?;
        let pos = record.pos + DbIndex::fixed_serialized_size() + offset;

        self.read_exact(pos, record.size)
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
    use super::*;
    use crate::test_utilities::test_file::TestFile;
    use crate::utilities::serialize::SerializeDynamicSized;
    use crate::utilities::serialize::SerializeFixedSized;

    #[test]
    fn insert_value() {
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
            Ok(i64::fixed_serialized_size() as u64)
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
}
