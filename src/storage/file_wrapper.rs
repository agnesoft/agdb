use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};

const ERROR_MESSAGE: &str = "Could not access file";

#[allow(dead_code)]
pub(crate) struct FileWrapper {
    pub(crate) file: File,
    pub(crate) filename: String,
    pub(crate) size: u64,
}

impl From<String> for FileWrapper {
    fn from(filename: String) -> Self {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(&filename)
            .expect(ERROR_MESSAGE);

        let size = file.seek(SeekFrom::End(0)).expect(ERROR_MESSAGE);

        FileWrapper {
            file,
            filename,
            size,
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
        let test_file = TestFile::from("./file_wrapper_test01.agdb");
        let file = FileWrapper::from(test_file.file_name().clone());

        assert!(Path::new(test_file.file_name()).exists());
        assert_eq!(&file.filename, test_file.file_name());
        assert_eq!(file.size, 0);
    }

    #[test]
    fn open_existing_file() {
        let test_file = TestFile::from("./file_storage_test02.agdb");
        File::create(test_file.file_name()).unwrap();
        let _storage = FileWrapper::from(test_file.file_name().clone());
    }
}
