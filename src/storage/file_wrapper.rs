use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

const ERROR_MESSAGE: &str = "Could not access file";

#[allow(dead_code)]
pub(crate) struct FileWrapper {
    pub(crate) file: File,
    pub(crate) filename: String,
    pub(crate) size: u64,
}

#[allow(dead_code)]
impl FileWrapper {
    pub(crate) fn current_pos(&mut self) -> u64 {
        self.file.seek(SeekFrom::Current(0)).expect(ERROR_MESSAGE)
    }

    pub(crate) fn read(&mut self, size: u64) -> Vec<u8> {
        let mut buffer = vec![0_u8; size as usize];
        self.file.read_exact(&mut buffer).expect(ERROR_MESSAGE);
        buffer
    }

    pub(crate) fn seek(&mut self, position: u64) {
        self.file
            .seek(SeekFrom::Start(position))
            .expect(ERROR_MESSAGE);
    }

    pub(crate) fn seek_end(&mut self) {
        self.file.seek(SeekFrom::End(0)).expect(ERROR_MESSAGE);
    }

    pub(crate) fn write(&mut self, data: &[u8]) {
        self.size = self.size + data.len() as u64;
        self.file.write_all(data).expect(ERROR_MESSAGE);
    }
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
    use crate::storage::serialize::Serialize;
    use crate::test_utilities::test_file::TestFile;
    use std::mem::size_of;
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
        let _file = FileWrapper::from(test_file.file_name().clone());
    }

    #[test]
    fn seek() {
        let test_file = TestFile::from("./file_storage_test03.agdb");
        let mut file = FileWrapper::from(test_file.file_name().clone());

        assert_eq!(file.current_pos(), 0);
        file.seek(10);
        assert_eq!(file.current_pos(), 10);
    }

    #[test]
    fn seek_end() {
        let test_file = TestFile::from("./file_storage_test03.agdb");
        let mut file = FileWrapper::from(test_file.file_name().clone());
        let data = 10_i64.serialize();
        file.write(&data);
        file.seek(0);

        assert_eq!(file.current_pos(), 0);
        file.seek_end();
        assert_eq!(file.current_pos(), size_of::<i64>() as u64);
    }

    #[test]
    fn write_read_bytes() {
        let test_file = TestFile::from("./file_storage_test04.agdb");
        let mut file = FileWrapper::from(test_file.file_name().clone());
        let data = 10_i64.serialize();

        assert_eq!(file.size, 0);

        file.write(&data);

        assert_eq!(file.size, size_of::<i64>() as u64);

        file.seek(0);

        let actual_data = file.read(size_of::<i64>() as u64);

        assert_eq!(data, actual_data);
    }
}
