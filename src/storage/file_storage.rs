use super::file_record::FileRecord;
use super::file_records::FileRecords;
use super::file_wrapper::FileWrapper;
use super::serialize::Serialize;
use std::io::Read;
use std::io::Write;
use std::io::{Seek, SeekFrom};
use std::mem::size_of;

#[allow(dead_code)]
pub(crate) struct FileStorage {
    file: FileWrapper,
    records: FileRecords,
}

#[allow(dead_code)]
impl FileStorage {
    pub(crate) fn insert<T: Serialize>(&mut self, value: &T) -> i64 {
        let position = self.file.file.seek(SeekFrom::End(0)).unwrap();
        let bytes = value.serialize();
        let record = self.records.create(position, bytes.len() as u64);
        self.file.file.write_all(&record.serialize()).unwrap();
        self.file.file.write_all(&bytes).unwrap();
        record.index
    }

    pub(crate) fn value<T: Serialize>(&mut self, index: i64) -> Option<T> {
        if let Some(record) = self.records.get(index) {
            self.file
                .file
                .seek(SeekFrom::Start(record.position + FileRecord::size() as u64))
                .unwrap();
            let mut bytes: Vec<u8> = vec![0; record.size as usize];
            self.file.file.read_exact(&mut bytes).unwrap();
            return Some(T::deserialize(&bytes));
        }

        None
    }

    pub(crate) fn value_at<T: Serialize>(&mut self, index: i64, offset: u64) -> Option<T> {
        if let Some(record) = self.records.get(index) {
            self.file
                .file
                .seek(SeekFrom::Start(
                    record.position + FileRecord::size() as u64 + offset,
                ))
                .unwrap();
            let mut bytes: Vec<u8> = vec![0; size_of::<T>()];
            self.file.file.read_exact(&mut bytes).unwrap();
            return Some(T::deserialize(&bytes));
        }

        None
    }
}

impl From<&str> for FileStorage {
    fn from(filename: &str) -> Self {
        FileStorage::from(filename.to_string())
    }
}

impl From<String> for FileStorage {
    fn from(filename: String) -> Self {
        FileStorage {
            file: FileWrapper::from(filename),
            records: FileRecords::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn insert() {
        let test_file = TestFile::from("./file_storage_test02.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());

        let index = storage.insert(&10_i64);

        assert_eq!(index, 0);
    }

    #[test]
    fn value() {
        let test_file = TestFile::from("./file_storage_test04.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());

        let index = storage.insert(&10_i64);

        assert_eq!(storage.value::<i64>(index), Some(10_i64));
    }

    #[test]
    fn value_at() {
        let test_file = TestFile::from("./file_storage_test05.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());
        let data = vec![1_i64, 2_i64, 3_i64];

        let index = storage.insert(&data);
        let offset = (size_of::<u64>() + size_of::<i64>()) as u64;

        assert_eq!(storage.value_at::<i64>(index, offset), Some(2_i64));
    }

    #[test]
    fn value_at_of_missing_index() {
        let test_file = TestFile::from("./file_storage_test06.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());
        assert_eq!(storage.value_at::<i64>(0, 8), None);
    }

    #[test]
    fn value_of_missing_index() {
        let test_file = TestFile::from("./file_storage_test07.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());
        assert_eq!(storage.value::<i64>(0), None);
    }
}
