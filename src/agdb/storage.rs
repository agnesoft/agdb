pub mod storage_file;
pub mod storage_index;

mod storage_data;
mod storage_data_file;
mod storage_impl;
mod storage_record;
mod storage_records;
mod write_ahead_log;

use crate::db_error::DbError;
use crate::storage::storage_index::StorageIndex;
use crate::utilities::serialize::Serialize;

pub trait Storage {
    fn commit(&mut self) -> Result<(), DbError>;
    fn insert<V: Serialize>(&mut self, value: &V) -> Result<StorageIndex, DbError>;
    fn insert_at<V: Serialize>(
        &mut self,
        index: &StorageIndex,
        offset: u64,
        value: &V,
    ) -> Result<(), DbError>;
    fn move_at(
        &mut self,
        index: &StorageIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError>;
    fn remove(&mut self, index: &StorageIndex) -> Result<(), DbError>;
    fn resize_value(&mut self, index: &StorageIndex, new_size: u64) -> Result<(), DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn size(&mut self) -> Result<u64, DbError>;
    fn transaction(&mut self);
    fn value<V: Serialize>(&mut self, index: &StorageIndex) -> Result<V, DbError>;
    fn value_at<V: Serialize>(&mut self, index: &StorageIndex, offset: u64) -> Result<V, DbError>;
    fn value_size(&self, index: &StorageIndex) -> Result<u64, DbError>;
}
