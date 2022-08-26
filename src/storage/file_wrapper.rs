use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};

use crate::db_error::DbError;

#[allow(dead_code)]
pub(crate) struct FileWrapper<FileT = File>
where
    FileT: Read,
    FileT: Seek,
    FileT: Write,
{
    pub(crate) file: FileT,
    pub(crate) filename: String,
    pub(crate) size: u64,
}

#[allow(dead_code)]
impl<FileT> FileWrapper<FileT>
where
    FileT: Read,
    FileT: Seek,
    FileT: Write,
{
    pub(crate) fn current_pos(&mut self) -> Result<u64, DbError> {
        Ok(self.file.seek(SeekFrom::Current(0))?)
    }

    pub(crate) fn read(&mut self, size: u64) -> Result<Vec<u8>, DbError> {
        let mut buffer = vec![0_u8; size as usize];
        self.file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub(crate) fn seek(&mut self, position: u64) -> Result<(), DbError> {
        self.file.seek(SeekFrom::Start(position))?;
        Ok(())
    }

    pub(crate) fn seek_end(&mut self) -> Result<(), DbError> {
        self.file.seek(SeekFrom::End(0))?;
        Ok(())
    }

    pub(crate) fn write(&mut self, data: &[u8]) -> Result<(), DbError> {
        let end_position = self.current_pos()? + data.len() as u64;

        if end_position > self.size {
            self.size = end_position;
        }

        Ok(self.file.write_all(data)?)
    }
}

impl TryFrom<String> for FileWrapper {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(&filename)?;

        let size = file.seek(SeekFrom::End(0))?;

        Ok(FileWrapper {
            file,
            filename,
            size,
        })
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
        let file = FileWrapper::try_from(test_file.file_name().clone()).unwrap();

        assert!(Path::new(test_file.file_name()).exists());
        assert_eq!(&file.filename, test_file.file_name());
        assert_eq!(file.size, 0);
    }

    #[test]
    fn open_existing_file() {
        let test_file = TestFile::from("./file_storage_test02.agdb");
        File::create(test_file.file_name()).unwrap();
        let _file = FileWrapper::try_from(test_file.file_name().clone()).unwrap();
    }

    #[test]
    fn seek() {
        let test_file = TestFile::from("./file_storage_test03.agdb");
        let mut file = FileWrapper::try_from(test_file.file_name().clone()).unwrap();

        assert_eq!(file.current_pos(), Ok(0));
        file.seek(10).unwrap();
        assert_eq!(file.current_pos(), Ok(10));
    }

    #[test]
    fn seek_end() {
        let test_file = TestFile::from("./file_storage_test04.agdb");
        let mut file = FileWrapper::try_from(test_file.file_name().clone()).unwrap();
        let data = 10_i64.serialize();
        file.write(&data).unwrap();
        file.seek(0).unwrap();

        assert_eq!(file.current_pos(), Ok(0));
        file.seek_end().unwrap();
        assert_eq!(file.current_pos(), Ok(size_of::<i64>() as u64));
    }

    #[test]
    fn size_writing_at_end() {
        let test_file = TestFile::from("./file_storage_test05.agdb");
        let mut file = FileWrapper::try_from(test_file.file_name().clone()).unwrap();
        let data = 10_i64.serialize();

        assert_eq!(file.size, 0);
        file.write(&data).unwrap();
        assert_eq!(file.size, size_of::<i64>() as u64);
    }

    #[test]
    fn size_write_within_current_size() {
        let test_file = TestFile::from("./file_storage_test06.agdb");
        let mut file = FileWrapper::try_from(test_file.file_name().clone()).unwrap();
        let data = 10_i64.serialize();

        assert_eq!(file.size, 0);
        file.write(&data).unwrap();
        file.seek(0).unwrap();
        file.write(&data).unwrap();
        assert_eq!(file.size, size_of::<i64>() as u64);
    }

    #[test]
    fn size_writing_over_end() {
        let test_file = TestFile::from("./file_storage_test07.agdb");
        let mut file = FileWrapper::try_from(test_file.file_name().clone()).unwrap();
        let data = 10_i64.serialize();

        assert_eq!(file.size, 0);

        file.write(&data).unwrap();
        file.seek((size_of::<i64>() as u64) / 2).unwrap();
        file.write(&data).unwrap();

        assert_eq!(
            file.size,
            size_of::<i64>() as u64 + (size_of::<i64>() as u64 / 2)
        );
    }

    #[test]
    fn write_read_bytes() {
        let test_file = TestFile::from("./file_storage_test08.agdb");
        let mut file = FileWrapper::try_from(test_file.file_name().clone()).unwrap();
        let data = 10_i64.serialize();

        file.write(&data).unwrap();
        file.seek(0).unwrap();

        let actual_data = file.read(size_of::<i64>() as u64);

        assert_eq!(actual_data, Ok(data));
    }

    #[test]
    fn bad_file() {
        let test_file = TestFile::from("./file_storage_test09.agdb");
        let mut file = FileWrapper {
            file: BadFile {},
            filename: test_file.file_name().clone(),
            size: 0,
        };

        assert!(file.current_pos().is_err());
        assert!(file.seek(0).is_err());
        assert!(file.read(0).is_err());
        assert!(file.write(&vec![0_u8; 0]).is_err());
    }
}
