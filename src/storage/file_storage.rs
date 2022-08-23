use super::file_record::FileRecord;
use super::file_records::FileRecords;
use super::serialize::Serialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::io::{Seek, SeekFrom};

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
}

impl From<&str> for FileStorage {
    fn from(filename: &str) -> Self {
        FileStorage::from(filename.to_string())
    }
}

impl From<String> for FileStorage {
    fn from(filename: String) -> Self {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(&filename)
            .unwrap();

        FileStorage {
            filename,
            file,
            records: FileRecords::default(),
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
    fn open_existing_file() {
        let test_file = TestFile::from("./file_storage_test02.agdb");
        File::create(test_file.file_name()).unwrap();
        let _storage = FileStorage::from(test_file.file_name().clone());
    }

    #[test]
    fn insert() {
        let test_file = TestFile::from("./file_storage_test03.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());

        let index = storage.insert(&10_i64);

        assert_eq!(index, 0);
    }

    #[test]
    fn value() {
        let test_file = TestFile::from("./file_storage_test03.agdb");
        let mut storage = FileStorage::from(test_file.file_name().clone());

        let index = storage.insert(&10_i64);

        assert_eq!(storage.value::<i64>(index), Some(10_i64));
    }
}
