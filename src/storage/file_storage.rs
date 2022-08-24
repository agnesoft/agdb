use super::file_record::FileRecord;
use super::file_records::FileRecords;
use super::serialize::Serialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::io::{Seek, SeekFrom};
use std::mem::size_of;

#[allow(dead_code)]
pub(crate) struct FileStorage {
    filename: String,
    file: File,
    records: FileRecords,
}

#[allow(dead_code)]
impl FileStorage {
    pub(crate) fn insert<T: Serialize>(&mut self, value: &T) -> i64 {
        let position = self.file.seek(SeekFrom::End(0)).unwrap();
        let bytes = value.serialize();
        let record = self.records.create(position, bytes.len() as u64);
        self.file.write_all(&record.serialize()).unwrap();
        self.file.write_all(&bytes).unwrap();
        record.index
    }

    pub(crate) fn value<T: Serialize>(&mut self, index: i64) -> Option<T> {
        if let Some(record) = self.records.get(index) {
            self.file
                .seek(SeekFrom::Start(record.position + FileRecord::size() as u64))
                .unwrap();
            let mut bytes: Vec<u8> = vec![0; record.size as usize];
            self.file.read_exact(&mut bytes).unwrap();
            return Some(T::deserialize(&bytes));
        }

        None
    }

    pub(crate) fn value_at<T: Serialize>(&mut self, index: i64, offset: u64) -> Option<T> {
        if let Some(record) = self.records.get(index) {
            self.file
                .seek(SeekFrom::Start(
                    record.position + FileRecord::size() as u64 + offset,
                ))
                .unwrap();
            let mut bytes: Vec<u8> = vec![0; size_of::<T>()];
            self.file.read_exact(&mut bytes).unwrap();
            return Some(T::deserialize(&bytes));
        }

        None
    }

    fn read_record(file: &mut File) -> FileRecord {
        let pos = file.seek(SeekFrom::Current(0)).unwrap();
        let mut data = [0; FileRecord::size()];
        file.read_exact(&mut data).unwrap();
        let mut record = FileRecord::deserialize(&data);
        record.position = pos;
        file.seek(SeekFrom::Current(record.size as i64)).unwrap();

        record
    }

    fn read_records(file: &mut File) -> Vec<FileRecord> {
        let mut records: Vec<FileRecord> = vec![];
        let end = file.seek(SeekFrom::End(0)).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        while file.seek(SeekFrom::Current(0)).unwrap() != end {
            records.push(Self::read_record(file));
        }

        records
    }

    fn seek(file: &mut File, position: SeekFrom) {
        file.seek(position).expect("");
    }
}

impl From<&str> for FileStorage {
    fn from(filename: &str) -> Self {
        FileStorage::from(filename.to_string())
    }
}

impl From<String> for FileStorage {
    fn from(filename: String) -> Self {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(&filename)
            .unwrap();

        let records = Self::read_records(&mut file);

        FileStorage {
            filename,
            file,
            records: FileRecords::from(records),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;
    use std::path::Path;

    #[test]
    fn create_new_file() {
        let test_file = TestFile::from("./file_storage_test01.agdb");
        let _storage = FileStorage::from(test_file.file_name().as_str());

        assert!(Path::new(test_file.file_name()).exists());
    }

    #[test]
    fn insert() {
        let test_file = TestFile::from("./file_storage_test02.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());

        let index = storage.insert(&10_i64);

        assert_eq!(index, 0);
    }

    #[test]
    fn open_existing_file() {
        let test_file = TestFile::from("./file_storage_test03.agdb");
        File::create(test_file.file_name()).unwrap();
        let _storage = FileStorage::from(test_file.file_name().clone());
    }

    #[test]
    fn restore_from_open_file() {
        let test_file = TestFile::from("./file_storage_test04.agdb");
        let value1 = vec![1_i64, 2_i64, 3_i64];
        let value2 = 64_u64;
        let value3 = vec![4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
        let index1;
        let index2;
        let index3;

        {
            let mut storage = FileStorage::from(test_file.file_name().clone());
            index1 = storage.insert(&value1);
            index2 = storage.insert(&value2);
            index3 = storage.insert(&value3);
        }

        let mut storage = FileStorage::from(test_file.file_name().clone());

        assert_eq!(storage.value::<Vec<i64>>(index1), Some(value1));
        assert_eq!(storage.value::<u64>(index2), Some(value2));
        assert_eq!(storage.value::<Vec<i64>>(index3), Some(value3));
    }

    #[test]
    fn value() {
        let test_file = TestFile::from("./file_storage_test05.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());

        let index = storage.insert(&10_i64);

        assert_eq!(storage.value::<i64>(index), Some(10_i64));
    }

    #[test]
    fn value_at() {
        let test_file = TestFile::from("./file_storage_test06.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());
        let data = vec![1_i64, 2_i64, 3_i64];

        let index = storage.insert(&data);
        let offset = (size_of::<u64>() + size_of::<i64>()) as u64;

        assert_eq!(storage.value_at::<i64>(index, offset), Some(2_i64));
    }

    #[test]
    fn value_at_of_missing_index() {
        let test_file = TestFile::from("./file_storage_test07.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());
        assert_eq!(storage.value_at::<i64>(0, 8), None);
    }

    #[test]
    fn value_of_missing_index() {
        let test_file = TestFile::from("./file_storage_test08.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());
        assert_eq!(storage.value::<i64>(0), None);
    }
}
