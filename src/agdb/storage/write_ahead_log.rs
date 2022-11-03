use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WriteAheadLogRecord {
    pub position: u64,
    pub bytes: Vec<u8>,
}

pub struct WriteAheadLog {
    file: File,
}

impl WriteAheadLog {
    pub fn clear(&mut self) -> Result<(), DbError> {
        Ok(self.file.set_len(0)?)
    }

    pub fn insert(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError> {
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&record.position.serialize())?;
        self.file
            .write_all(&(record.bytes.len() as u64).serialize())?;
        self.file.write_all(&record.bytes)?;

        Ok(())
    }

    pub fn records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError> {
        let mut records = Vec::<WriteAheadLogRecord>::new();
        let size = self.file.seek(SeekFrom::End(0))?;
        self.file.seek(SeekFrom::Start(0))?;

        while self.file.seek(SeekFrom::Current(0))? < size {
            records.push(Self::read_record(&mut self.file)?);
        }

        Ok(records)
    }

    fn read_exact(file: &mut File, size: u64) -> Result<Vec<u8>, DbError> {
        let mut buffer = vec![0_u8; size as usize];
        file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn read_record(file: &mut File) -> Result<WriteAheadLogRecord, DbError> {
        let position = u64::deserialize(&Self::read_exact(file, u64::serialized_size())?)?;
        let size = u64::deserialize(&Self::read_exact(file, u64::serialized_size())?)?;

        Ok(WriteAheadLogRecord {
            position,
            bytes: Self::read_exact(file, size)?,
        })
    }

    pub(crate) fn wal_filename(filename: &str) -> String {
        let pos;

        if let Some(slash) = filename.rfind('/') {
            pos = slash + 1;
        } else if let Some(backslash) = filename.rfind('\\') {
            pos = backslash + 1
        } else {
            pos = 0;
        }

        let mut copy = filename.to_owned();
        copy.insert(pos, '.');
        copy
    }
}

impl TryFrom<&String> for WriteAheadLog {
    type Error = DbError;

    fn try_from(filename: &String) -> Result<Self, Self::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(WriteAheadLog::wal_filename(filename))?;

        Ok(WriteAheadLog { file })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn clear() {
        let test_file = TestFile::new();

        let mut wal = WriteAheadLog::try_from(test_file.file_name()).unwrap();
        let record = WriteAheadLogRecord {
            position: 1,
            bytes: vec![1_u8; 5],
        };

        wal.insert(record).unwrap();
        wal.clear().unwrap();

        assert_eq!(wal.records(), Ok(vec![]));
    }

    #[test]
    fn filename_constructed() {
        let test_file = TestFile::new();
        WriteAheadLog::try_from(test_file.file_name()).unwrap();
    }

    #[test]
    fn insert() {
        let test_file = TestFile::from(".\\write_ahead_log_test.rs-insert.testfile");

        let mut wal = WriteAheadLog::try_from(test_file.file_name()).unwrap();
        let record = WriteAheadLogRecord {
            position: 1,
            bytes: vec![1_u8; 5],
        };

        wal.insert(record.clone()).unwrap();

        assert_eq!(wal.records(), Ok(vec![record]));
    }

    #[test]
    fn insert_empty() {
        let test_file = TestFile::from("./write_ahead_log_test.rs-insert_empty.testfile");

        let mut wal = WriteAheadLog::try_from(test_file.file_name()).unwrap();
        let record = WriteAheadLogRecord {
            position: 16,
            bytes: vec![],
        };

        wal.insert(record.clone()).unwrap();

        assert_eq!(wal.records(), Ok(vec![record]));
    }

    #[test]
    fn record_derived_from_debug() {
        let record = WriteAheadLogRecord::default();
        format!("{:?}", record);
    }

    #[test]
    fn records() {
        let test_file = TestFile::from("write_ahead_log_test.rs-records.testfile");

        let mut wal = WriteAheadLog::try_from(test_file.file_name()).unwrap();
        let record1 = WriteAheadLogRecord {
            position: 1,
            bytes: vec![1_u8; 5],
        };
        let record2 = WriteAheadLogRecord {
            position: 15,
            bytes: vec![2_u8; 3],
        };

        wal.insert(record1.clone()).unwrap();
        wal.insert(record2.clone()).unwrap();

        assert_eq!(wal.records(), Ok(vec![record1, record2]));
    }
}
