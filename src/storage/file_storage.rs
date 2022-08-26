use super::file_record::FileRecord;
use super::file_records::FileRecords;
use super::file_wrapper::FileWrapper;
use super::serialize::Serialize;
use crate::db_error::DbError;
use std::mem::size_of;

#[allow(dead_code)]
pub(crate) struct FileStorage {
    file: FileWrapper,
    records: FileRecords,
}

#[allow(dead_code)]
impl FileStorage {
    pub(crate) fn insert<T: Serialize>(&mut self, value: &T) -> Result<i64, DbError> {
        self.file.seek_end()?;
        let bytes = value.serialize();
        let record = self.records.create(self.file.size, bytes.len() as u64);
        self.file.write(&record.serialize())?;
        self.file.write(&bytes)?;
        Ok(record.index)
    }

    pub(crate) fn value<T: Serialize>(&mut self, index: i64) -> Result<T, DbError> {
        if let Some(record) = self.records.get(index) {
            self.file
                .seek(record.position + FileRecord::size() as u64)?;
            return T::deserialize(&self.file.read(record.size)?);
        }

        Err(DbError::Storage(format!("index '{}' not found", index)))
    }

    pub(crate) fn value_at<T: Serialize>(&mut self, index: i64, offset: u64) -> Result<T, DbError> {
        if let Some(record) = self.records.get(index) {
            let read_start = record.position + FileRecord::size() as u64 + offset;
            self.file.seek(read_start)?;
            return T::deserialize(&self.file.read(size_of::<T>() as u64)?);
        }

        Err(DbError::Storage(format!("index '{}' not found", index)))
    }

    fn read_record(file: &mut FileWrapper) -> Result<FileRecord, DbError> {
        let pos = file.current_pos()?;
        let mut record = FileRecord::deserialize(&file.read(FileRecord::size() as u64)?)?;
        record.position = pos;
        file.seek(pos + FileRecord::size() as u64 + record.size)?;

        Ok(record)
    }

    fn read_records(file: &mut FileWrapper) -> Result<Vec<FileRecord>, DbError> {
        let mut records: Vec<FileRecord> = vec![];
        file.seek(0)?;

        while file.current_pos()? != file.size {
            records.push(Self::read_record(file)?);
        }

        Ok(records)
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
        let mut file = FileWrapper::try_from(filename)?;
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
        let test_file = TestFile::from("./file_storage_test01.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().as_str()).unwrap();

        let index = storage.insert(&10_i64);

        assert_eq!(index, Ok(0));
    }

    #[test]
    fn restore_from_open_file() {
        let test_file = TestFile::from("./file_storage_test02.agdb");
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
    fn value() {
        let test_file = TestFile::from("./file_storage_test03.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(storage.value::<i64>(index), Ok(10_i64));
    }

    #[test]
    fn value_at() {
        let test_file = TestFile::from("./file_storage_test04.agdb");

        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();
        let data = vec![1_i64, 2_i64, 3_i64];

        let index = storage.insert(&data).unwrap();
        let offset = (size_of::<u64>() + size_of::<i64>()) as u64;

        assert_eq!(storage.value_at::<i64>(index, offset), Ok(2_i64));
    }

    #[test]
    fn value_at_of_missing_index() {
        let test_file = TestFile::from("./file_storage_test05.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.value_at::<i64>(0, 8),
            Err(DbError::Storage("index '0' not found".to_string()))
        );
    }

    #[test]
    fn value_of_missing_index() {
        let test_file = TestFile::from("./file_storage_test06.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(
            storage.value::<i64>(0),
            Err(DbError::Storage("index '0' not found".to_string()))
        );
    }
    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::from("./file_storage_test07.agdb");
        let mut storage = FileStorage::try_from(test_file.file_name().clone()).unwrap();

        let index = storage.insert(&10_i64).unwrap();

        assert_eq!(
            storage.value::<Vec<i64>>(index),
            Err(DbError::Storage(
                "i64 deserialization error: out of bounds".to_string()
            ))
        );
    }
}
