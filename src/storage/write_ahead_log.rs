use super::serialize::Serialize;
use super::write_ahead_log_record::WriteAheadLogRecord;
use crate::db_error::DbError;
use std::io::Write;

#[allow(dead_code)]
pub(crate) struct WriteAheadLog {
    file: std::fs::File,
}

#[allow(dead_code)]
impl WriteAheadLog {
    pub(crate) fn insert(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError> {
        self.file.write_all(&record.position.serialize())?;
        self.file
            .write_all(&(record.bytes.len() as u64).serialize())?;
        self.file.write_all(&record.bytes)?;
        Ok(())
    }
}

impl TryFrom<String> for WriteAheadLog {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&filename)?;

        Ok(WriteAheadLog { file })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn filename_constructed() {
        let test_file = TestFile::from("./write_ahead_log-filename_constructed.agdb");
        WriteAheadLog::try_from(test_file.file_name().clone()).unwrap();
    }

    #[test]
    fn insert() {
        let test_file = TestFile::from("./write_ahead_log-insert.agdb");
        {
            let mut wal = WriteAheadLog::try_from(test_file.file_name().clone()).unwrap();
            wal.insert(WriteAheadLogRecord {
                position: 1,
                bytes: vec![1_u8; 5],
            })
            .unwrap();
        }

        let actual_size = std::fs::metadata(test_file.file_name()).unwrap().len();
        let expected_size = (std::mem::size_of::<u64>() + 5) as u64;

        assert_eq!(actual_size, expected_size);
    }
}
