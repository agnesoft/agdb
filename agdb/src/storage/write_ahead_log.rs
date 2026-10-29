use crate::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

/// The `len` covers only fully written records. Bytes past it are the
/// remains of a failed insert (`torn`) and are cut off before the next
/// insert so that the log never contains a partial record followed by
/// valid ones.
#[derive(Debug)]
pub struct WriteAheadLog {
    file: File,
    len: u64,
    torn: bool,
}

pub struct WriteAheadLogRecord {
    pub pos: u64,
    pub value: Vec<u8>,
}

impl WriteAheadLog {
    pub fn clear(&mut self) -> Result<(), DbError> {
        self.file.set_len(0)?;
        self.len = 0;
        self.torn = false;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), DbError> {
        Ok(self.file.sync_data()?)
    }

    pub fn insert(&mut self, pos: u64, value: &[u8]) -> Result<(), DbError> {
        self.truncate_torn()?;

        let mut record = pos.serialize();
        record.extend((value.len() as u64).serialize());
        record.extend_from_slice(value);

        self.file.seek(SeekFrom::Start(self.len))?;

        if let Err(error) = self.file.write_all(&record) {
            self.torn = true;
            let _ = self.truncate_torn();
            return Err(error.into());
        }

        self.len += record.len() as u64;
        Ok(())
    }

    pub fn new(filename: &str) -> Result<WriteAheadLog, DbError> {
        let mut wal = Self {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .create(true)
                .open(WriteAheadLog::wal_filename(filename))?,
            len: 0,
            torn: false,
        };

        wal.repair()?;

        Ok(wal)
    }

    pub fn records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError> {
        let mut records = Vec::<WriteAheadLogRecord>::new();
        let size = self.len;
        self.file.rewind()?;

        while self.file.stream_position()? < size {
            records.push(Self::read_record(&mut self.file)?);
        }

        Ok(records)
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

        let mut name = filename.to_owned();
        name.insert(pos, '.');

        name
    }

    fn read_exact(file: &mut File, size: u64) -> Result<Vec<u8>, DbError> {
        let mut buffer = vec![0_u8; size as usize];
        file.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    fn read_record(file: &mut File) -> Result<WriteAheadLogRecord, DbError> {
        let pos = u64::deserialize(&Self::read_exact(file, u64::serialized_size_static())?)?;
        let size = u64::deserialize(&Self::read_exact(file, u64::serialized_size_static())?)?;

        Ok(WriteAheadLogRecord {
            pos,
            value: Self::read_exact(file, size)?,
        })
    }

    fn skip_record(file: &mut File) -> Result<(), DbError> {
        file.seek(SeekFrom::Current(u64::serialized_size_static() as i64))?;
        let value_size = u64::deserialize(&Self::read_exact(file, u64::serialized_size_static())?)?;
        file.seek(SeekFrom::Current(value_size as i64))?;
        Ok(())
    }

    fn repair(&mut self) -> Result<(), DbError> {
        let size = self.file.seek(SeekFrom::End(0))?;
        self.file.rewind()?;
        let mut pos = 0_u64;

        while pos < size {
            if Self::skip_record(&mut self.file).is_err() {
                break;
            }

            let new_pos = self.file.stream_position()?;

            if new_pos > size {
                break;
            }

            pos = new_pos;
        }

        if pos < size {
            self.file.set_len(pos)?;
        }

        self.len = pos;
        Ok(())
    }

    fn truncate_torn(&mut self) -> Result<(), DbError> {
        if self.torn {
            self.file.set_len(self.len)?;
            self.torn = false;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn records_ignore_bytes_past_len() {
        let test_file = TestFile::new();
        let mut wal = WriteAheadLog::new(test_file.file_name()).unwrap();
        wal.insert(1, &[1, 2]).unwrap();
        OpenOptions::new()
            .append(true)
            .open(WriteAheadLog::wal_filename(test_file.file_name()))
            .unwrap()
            .write_all(&[0; 16])
            .unwrap();

        let records = wal.records().unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].pos, 1);
        assert_eq!(records[0].value, vec![1, 2]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn failed_insert_is_cut_off_before_next_insert() {
        let test_file = TestFile::new();
        let mut wal = WriteAheadLog::new(test_file.file_name()).unwrap();
        wal.insert(1, &[1, 2]).unwrap();
        let file = std::mem::replace(
            &mut wal.file,
            OpenOptions::new().write(true).open("/dev/full").unwrap(),
        );

        assert!(wal.insert(2, &[3, 4]).is_err());
        assert!(wal.insert(3, &[5, 6]).is_err());

        wal.file = file;
        wal.file.seek(SeekFrom::End(0)).unwrap();
        wal.file.write_all(&[2, 0, 0, 0, 0, 0, 0, 0, 2]).unwrap();
        wal.insert(4, &[7, 8]).unwrap();
        let records = wal.records().unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[1].pos, 4);
        assert_eq!(records[1].value, vec![7, 8]);
        assert_eq!(wal.file.seek(SeekFrom::End(0)).unwrap(), wal.len);
    }

    #[test]
    fn wal_filename() {
        assert_eq!(WriteAheadLog::wal_filename("file"), ".file");
        assert_eq!(WriteAheadLog::wal_filename("/file"), "/.file");
        assert_eq!(
            WriteAheadLog::wal_filename("\\some\\path\\file"),
            "\\some\\path\\.file"
        );
    }
}
