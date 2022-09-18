mod file_storage;
mod serialize;
mod stable_hash;
mod storage_hash_map;
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

        self.append(&index.serialize())?;
        self.append(&(bytes.len() as u64).serialize())?;
        self.append(&bytes)?;
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
        self.ensure_record_size(&mut record, index, offset, bytes.len() as u64)?;
        self.write(Self::value_position(record.position, offset), &bytes)?;
        self.commit()
    }

    fn move_at(
        &mut self,
        index: i64,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError> {
        if offset_from == offset_to || size == 0 {
            return Ok(());
        }

        let mut record = self.record(index)?;
        Self::validate_move_size(offset_from, size, record.size)?;
        self.transaction();
        self.ensure_record_size(&mut record, index, offset_to, size)?;
        self.move_bytes(
            Self::value_position_u64(record.position, offset_from),
            Self::value_position_u64(record.position, offset_to),
            size,
        )?;
        self.commit()?;

        Ok(())
    }

    fn remove(&mut self, index: i64) -> Result<(), DbError> {
        self.transaction();
        let position = self.record(index)?.position;
        self.invalidate_record(index, position)?;
        self.remove_index(index);
        self.commit()
    }

    fn resize_value(&mut self, index: i64, new_size: u64) -> Result<(), DbError> {
        if new_size == 0 {
            return Err(DbError::from("value size cannot be 0"));
        }

        let mut record = self.record(index)?;

        if record.size != new_size {
            self.transaction();
            self.resize_record(index, new_size, new_size, &mut record)?;
            self.commit()?;
        }

        Ok(())
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
