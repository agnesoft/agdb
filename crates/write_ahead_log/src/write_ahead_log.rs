use crate::write_ahead_log_record::WriteAheadLogRecord;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

pub struct WriteAheadLog {
    file: std::fs::File,
}

impl WriteAheadLog {
    pub fn clear(&mut self) -> Result<(), DbError> {
        Ok(self.file.set_len(0)?)
    }

    pub fn insert(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError> {
        self.file.seek(std::io::SeekFrom::End(0))?;
        self.file.write_all(&record.position.serialize())?;
        self.file
            .write_all(&(record.bytes.len() as u64).serialize())?;
        self.file.write_all(&record.bytes)?;

        Ok(())
    }

    pub fn records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError> {
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
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(WriteAheadLog::wal_filename(filename))?;

        Ok(WriteAheadLog { file })
    }
}
