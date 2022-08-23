use super::file_records::FileRecords;
use std::fs::{File, OpenOptions};

#[allow(dead_code)]
pub(crate) struct FileStorage {
    filename: String,
    file: File,
    records: FileRecords,
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
}
