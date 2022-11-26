pub mod file_storage;
pub mod storage_index;
pub mod storage_value;

mod file_record;
mod file_records;
mod write_ahead_log;

use self::storage_index::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::DbError;

pub trait Storage {
    fn commit(&mut self) -> Result<(), DbError>;
    fn insert<T: Serialize>(&mut self, value: &T) -> Result<StorageIndex, DbError>;
    fn insert_at<T: Serialize>(
        &mut self,
        index: &StorageIndex,
        offset: u64,
        value: &T,
    ) -> Result<(), DbError>;
    fn insert_bytes(&mut self, bytes: &[u8]) -> Result<StorageIndex, DbError>;
    fn insert_bytes_at(
        &mut self,
        index: &StorageIndex,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), DbError>;
    fn len(&self) -> Result<u64, DbError>;
    fn move_at(
        &mut self,
        index: &StorageIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError>;
    fn remove(&mut self, index: &StorageIndex) -> Result<(), DbError>;
    fn replace<T: Serialize>(&mut self, index: &StorageIndex, value: &T) -> Result<(), DbError>;
    fn replace_with_bytes(&mut self, index: &StorageIndex, bytes: &[u8]) -> Result<(), DbError>;
    fn resize_value(&mut self, index: &StorageIndex, new_size: u64) -> Result<(), DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn transaction(&mut self);
    fn value<T: Serialize>(&self, index: &StorageIndex) -> Result<T, DbError>;
    fn value_as_bytes(&self, index: &StorageIndex) -> Result<Vec<u8>, DbError>;
    fn value_as_bytes_at(&self, index: &StorageIndex, offset: u64) -> Result<Vec<u8>, DbError>;
    fn value_as_bytes_at_size(
        &self,
        index: &StorageIndex,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, DbError>;
    fn value_at<T: Serialize>(&self, index: &StorageIndex, offset: u64) -> Result<T, DbError>;
    fn value_size(&self, index: &StorageIndex) -> Result<u64, DbError>;
}
