use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize_static::SerializeStatic;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

pub struct WriteAheadLog {
    file: File,
}

pub struct WriteAheadLogRecord {
    pub pos: u64,
    pub value: Vec<u8>,
}

impl WriteAheadLog {
    pub fn clear(&mut self) -> Result<(), DbError> {
        Ok(self.file.set_len(0)?)
    }

    pub fn insert(&mut self, pos: u64, value: Vec<u8>) -> Result<(), DbError> {
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&pos.serialize())?;
        self.file.write_all(&value.serialize())?;

        Ok(())
    }

    pub fn new(filename: &str) -> Result<WriteAheadLog, DbError> {
        Ok(Self {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(WriteAheadLog::wal_filename(filename))?,
        })
    }

    pub fn records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError> {
        let mut records = Vec::<WriteAheadLogRecord>::new();
        let size = self.file.seek(SeekFrom::End(0))?;
        self.file.rewind()?;

        while self.file.stream_position()? < size {
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
        let pos = u64::deserialize(&Self::read_exact(file, u64::static_serialized_size())?)?;
        let size = u64::deserialize(&Self::read_exact(file, u64::static_serialized_size())?)?;

        Ok(WriteAheadLogRecord {
            pos,
            value: Self::read_exact(file, size)?,
        })
    }

    fn wal_filename(filename: &str) -> String {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
