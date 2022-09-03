use std::io::Read;
use std::io::Seek;
use std::io::Write;

use super::file_record::FileRecord;
use super::file_record_full::FileRecordFull;
use super::file_records::FileRecords;
use super::serialize::Serialize;
use crate::db_error::DbError;

#[allow(dead_code)]
pub(crate) struct FileStorage {
    file: std::fs::File,
    records: FileRecords,
}

#[allow(dead_code)]
impl FileStorage {
    pub(crate) fn insert<T: Serialize>(&mut self, value: &T) -> Result<i64, DbError> {
        let position = self.size()?;
        let bytes = value.serialize();
        let index = self.records.create(position, bytes.len() as u64);
        self.append(index.serialize())?;
        self.append((bytes.len() as u64).serialize())?;
        self.append(bytes)?;
        Ok(index)
    }

    pub(crate) fn insert_at<T: Serialize>(
        &mut self,
        index: i64,
        offset: u64,
        value: &T,
    ) -> Result<(), DbError> {
        let mut record = self.record(index)?;
        let bytes = T::serialize(value);
        let new_size = offset + bytes.len() as u64;

        if new_size > record.size {
            self.move_record_to_end(index, new_size, offset, &mut record)?;
        }

        let write_start = record.position + std::mem::size_of::<FileRecord>() as u64 + offset;
        self.write(std::io::SeekFrom::Start(write_start), bytes)?;
        return Ok(());
    }

    pub(crate) fn remove(&mut self, index: i64) -> Result<(), DbError> {
        let position = self.record_position(index)?;
        self.write(std::io::SeekFrom::Start(position), (-index).serialize())?;
        self.records.remove(index);
        Ok(())
    }

    pub(crate) fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        let indexes = self.records.indexes_by_position();
        let mut current_pos = self.file.seek(std::io::SeekFrom::Start(0))?;

        for index in indexes {
            if let Some(record) = self.records.get_mut(index) {
                if record.position != current_pos {
                    self.file.seek(std::io::SeekFrom::Start(record.position))?;
                    let bytes = FileStorage::read_exact(
                        &mut self.file,
                        std::mem::size_of::<FileRecord>() as u64 + record.size,
                    )?;
                    record.position = current_pos;
                    self.write(std::io::SeekFrom::Start(current_pos), bytes)?;
                } else {
                    self.file.seek(std::io::SeekFrom::Current(
                        std::mem::size_of::<FileRecord>() as i64 + record.size as i64,
                    ))?;
                }
            }

            current_pos = self.file.seek(std::io::SeekFrom::Current(0))?;
        }

        self.file.set_len(current_pos)?;

        Ok(())
    }

    pub(crate) fn value<T: Serialize>(&mut self, index: i64) -> Result<T, DbError> {
        if let Some(record) = self.records.get(index) {
            let value_pos = record.position + std::mem::size_of::<FileRecord>() as u64;
            self.file.seek(std::io::SeekFrom::Start(value_pos))?;
            return T::deserialize(&FileStorage::read_exact(&mut self.file, record.size)?);
        }

        Err(DbError::Storage(format!("index '{}' not found", index)))
    }

    pub(crate) fn value_at<T: Serialize>(&mut self, index: i64, offset: u64) -> Result<T, DbError> {
        if let Some(record) = self.records.get(index) {
            let read_start = record.position + std::mem::size_of::<FileRecord>() as u64 + offset;
            let value_size = FileStorage::value_read_size::<T>(record.size, offset)?;
            self.file.seek(std::io::SeekFrom::Start(read_start))?;
            return T::deserialize(&FileStorage::read_exact(&mut self.file, value_size)?);
        }

        Err(DbError::Storage(format!("index '{}' not found", index)))
    }

    pub(crate) fn value_size(&self, index: i64) -> Result<u64, DbError> {
        if let Some(record) = self.records.get(index) {
            return Ok(record.size);
        }

        Err(DbError::Storage(format!("index '{}' not found", index)))
    }

    fn append(&mut self, bytes: Vec<u8>) -> Result<(), DbError> {
        self.write(std::io::SeekFrom::End(0), bytes)
    }

    fn copy_record_to_end(
        &mut self,
        from: u64,
        size: u64,
        record_index: i64,
        record_size: u64,
    ) -> Result<FileRecord, DbError> {
        let new_position = self.size()?;
        self.file.seek(std::io::SeekFrom::Start(from))?;
        let orig_value = FileStorage::read_exact(&mut self.file, size)?;
        self.append(record_index.serialize())?;
        self.append(record_size.serialize())?;
        self.append(orig_value)?;
        Ok(FileRecord {
            position: new_position,
            size: record_size,
        })
    }

    fn move_record_to_end(
        &mut self,
        index: i64,
        new_size: u64,
        offset: u64,
        record: &mut FileRecord,
    ) -> Result<(), DbError> {
        *record = self.copy_record_to_end(
            record.position + std::mem::size_of::<FileRecord>() as u64,
            core::cmp::min(record.size, offset),
            index,
            new_size,
        )?;
        *self.record_mut(index)? = record.clone();
        Ok(())
    }

    fn read_exact(file: &mut std::fs::File, size: u64) -> Result<Vec<u8>, DbError> {
        let mut buffer = vec![0_u8; size as usize];
        file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn record_position(&mut self, index: i64) -> Result<u64, DbError> {
        Ok(self.record(index)?.position)
    }

    fn record(&mut self, index: i64) -> Result<FileRecord, DbError> {
        Ok(self
            .records
            .get(index)
            .ok_or(DbError::Storage(format!("index '{}' not found", index)))?
            .clone())
    }

    fn record_mut(&mut self, index: i64) -> Result<&mut FileRecord, DbError> {
        Ok(self
            .records
            .get_mut(index)
            .ok_or(DbError::Storage(format!("index '{}' not found", index)))?)
    }

    fn read_record(file: &mut std::fs::File) -> Result<FileRecordFull, DbError> {
        let position = file.seek(std::io::SeekFrom::Current(0))?;
        let index = i64::deserialize(&FileStorage::read_exact(
            file,
            std::mem::size_of::<i64>() as u64,
        )?)?;
        let size = u64::deserialize(&FileStorage::read_exact(
            file,
            std::mem::size_of::<u64>() as u64,
        )?)?;

        file.seek(std::io::SeekFrom::Current(size as i64))?;

        Ok(FileRecordFull {
            index,
            position,
            size,
        })
    }

    fn read_records(file: &mut std::fs::File) -> Result<Vec<FileRecordFull>, DbError> {
        let mut records: Vec<FileRecordFull> = vec![];
        file.seek(std::io::SeekFrom::End(0))?;
        let size = file.seek(std::io::SeekFrom::Current(0))?;
        file.seek(std::io::SeekFrom::Start(0))?;

        while file.seek(std::io::SeekFrom::Current(0))? < size {
            records.push(Self::read_record(file)?);
        }

        Ok(records)
    }

    fn size(&mut self) -> Result<u64, DbError> {
        Ok(self.file.seek(std::io::SeekFrom::End(0))?)
    }

    fn value_read_size<T>(size: u64, offset: u64) -> Result<u64, DbError> {
        let value_size = std::mem::size_of::<T>() as u64;

        if offset > size {
            return Err(DbError::Storage(format!(
                "{} deserialization error: offset out of bounds",
                std::any::type_name::<T>()
            )));
        }

        if size - offset < value_size {
            return Err(DbError::Storage(format!(
                "{} deserialization error: value out of bounds",
                std::any::type_name::<T>()
            )));
        }

        Ok(std::mem::size_of::<T>() as u64)
    }

    fn write(&mut self, position: std::io::SeekFrom, bytes: Vec<u8>) -> Result<(), DbError> {
        self.file.seek(position)?;
        Ok(self.file.write_all(&bytes)?)
    }
}

impl TryFrom<&str> for FileStorage {
    type Error = DbError;

    fn try_from(filename: &str) -> Result<Self, Self::Error> {
        FileStorage::try_from(filename.to_string())
    }
}

impl TryFrom<String> for FileStorage {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(&filename)?;
        let records = FileRecords::from(Self::read_records(&mut file)?);

        Ok(FileStorage { file, records })
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
                "i64 deserialization error: value out of bounds".to_string()
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
                "i64 deserialization error: offset out of bounds".to_string()
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
