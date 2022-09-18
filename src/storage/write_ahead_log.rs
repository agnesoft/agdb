use super::serialize::Serialize;
use super::write_ahead_log_record::WriteAheadLogRecord;
use crate::db_error::DbError;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

#[allow(dead_code)]
pub(crate) struct WriteAheadLog {
    file: std::fs::File,
}

#[allow(dead_code)]
impl WriteAheadLog {
    pub(crate) fn clear(&mut self) -> Result<(), DbError> {
        Ok(self.file.set_len(0)?)
    }

    pub(crate) fn insert(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError> {
        self.file.seek(std::io::SeekFrom::End(0))?;
        self.file.write_all(&record.position.serialize())?;
        self.file
            .write_all(&(record.bytes.len() as u64).serialize())?;
        self.file.write_all(&record.bytes)?;

        Ok(())
    }

    pub(crate) fn records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError> {
        let mut records = Vec::<WriteAheadLogRecord>::new();
        let size = self.file.seek(std::io::SeekFrom::End(0))?;
        self.file.seek(std::io::SeekFrom::Start(0))?;

        while self.file.seek(std::io::SeekFrom::Current(0))? < size {
            records.push(Self::read_record(&mut self.file)?);
        }

        Ok(records)
    }

    fn read_exact(file: &mut std::fs::File, size: u64) -> Result<Vec<u8>, DbError> {
        let mut buffer = vec![0_u8; size as usize];
        file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn read_record(file: &mut std::fs::File) -> Result<WriteAheadLogRecord, DbError> {
        let position = u64::deserialize(&Self::read_exact(file, u64::serialized_size() as u64)?)?;
        let size = u64::deserialize(&Self::read_exact(file, u64::serialized_size() as u64)?)?;

        Ok(WriteAheadLogRecord {
            position,
            bytes: Self::read_exact(file, size)?,
        })
    }
}

impl TryFrom<&String> for WriteAheadLog {
    type Error = DbError;

    fn try_from(filename: &String) -> Result<Self, Self::Error> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)?;

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
        WriteAheadLog::try_from(test_file.file_name()).unwrap();
    }

    #[test]
    fn clear() {
        let test_file = TestFile::from("./write_ahead_log-clear.agdb");

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
    fn insert() {
        let test_file = TestFile::from("./write_ahead_log-insert.agdb");

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
        let test_file = TestFile::from("./write_ahead_log-insert_empty.agdb");

        let mut wal = WriteAheadLog::try_from(test_file.file_name()).unwrap();
        let record = WriteAheadLogRecord {
            position: 16,
            bytes: vec![],
        };

        wal.insert(record.clone()).unwrap();

        assert_eq!(wal.records(), Ok(vec![record]));
    }

    #[test]
    fn records() {
        let test_file = TestFile::from("./write_ahead_log-records.agdb");

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
