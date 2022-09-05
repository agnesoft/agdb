mod file_storage;
mod serialize;
mod storage_impl;
mod storage_record;
mod storage_record_with_index;
mod storage_records;
mod storage_vec;
mod write_ahead_log;
mod write_ahead_log_record;

use self::serialize::Serialize;
use self::storage_impl::StorageImpl;
use crate::db_error::DbError;

pub(crate) trait Storage<T: StorageImpl = Self>: StorageImpl<T> {
    fn commit(&mut self) -> Result<(), DbError> {
        if self.end_transaction() {
            self.clear_wal()?;
        }

        Ok(())
    }

    fn insert<V: Serialize>(&mut self, value: &V) -> Result<i64, DbError> {
        self.transaction();
        let position = self.size()?;
        let bytes = value.serialize();
        let index = self.create_index(position, bytes.len() as u64);

        self.append(index.serialize())?;
        self.append((bytes.len() as u64).serialize())?;
        self.append(bytes)?;
        self.commit()?;

        Ok(index)
    }

    fn insert_at<V: Serialize>(
        &mut self,
        index: i64,
        offset: u64,
        value: &V,
    ) -> Result<(), DbError> {
        self.transaction();
        let mut record = self.record(index)?;
        let bytes = V::serialize(value);
        self.ensure_record_size(&mut record, index, offset, bytes.len())?;
        self.write(Self::value_position(record.position, offset), bytes)?;
        self.commit()
    }

    fn remove(&mut self, index: i64) -> Result<(), DbError> {
        self.transaction();
        let position = self.record(index)?.position;
        self.write(std::io::SeekFrom::Start(position), (-index).serialize())?;
        self.remove_index(index);
        self.commit()
    }

    fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.transaction();
        let indexes = self.indexes_by_position();
        let size = self.shrink_indexes(indexes)?;
        self.truncate(size)?;
        self.commit()
    }

    fn size(&mut self) -> Result<u64, DbError> {
        self.seek(std::io::SeekFrom::End(0))
    }

    fn transaction(&mut self) {
        self.begin_transaction();
    }

    fn value<V: Serialize>(&mut self, index: i64) -> Result<V, DbError> {
        let record = self.record(index)?;
        V::deserialize(&self.read(Self::value_position(record.position, 0), record.size)?)
    }

    fn value_at<V: Serialize>(&mut self, index: i64, offset: u64) -> Result<V, DbError> {
        let record = self.record(index)?;
        let bytes = self.read(
            Self::value_position(record.position, offset),
            Self::value_read_size::<V>(record.size, offset)?,
        );

        V::deserialize(&bytes?)
    }

    fn value_size(&self, index: i64) -> Result<u64, DbError> {
        Ok(self.record(index)?.size)
    }
}
