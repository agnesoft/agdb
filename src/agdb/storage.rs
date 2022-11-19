pub mod file_storage;

mod file_record;
mod file_records;
mod file_storage_impl;
mod write_ahead_log;

use crate::utilities::serialize::Serialize;
use crate::DbError;
use crate::DbIndex;

pub trait Storage {
    fn commit(&mut self) -> Result<(), DbError>;
    fn insert<T: Serialize>(&mut self, value: &T) -> Result<DbIndex, DbError>;
    fn insert_at<T: Serialize>(
        &mut self,
        index: &DbIndex,
        offset: u64,
        value: &T,
    ) -> Result<u64, DbError>;
    fn insert_bytes(&mut self, bytes: &[u8]) -> Result<DbIndex, DbError>;
    fn insert_bytes_at(
        &mut self,
        index: &DbIndex,
        offset: u64,
        bytes: &[u8],
    ) -> Result<u64, DbError>;
    fn len(&self) -> Result<u64, DbError>;
    fn move_at(
        &mut self,
        index: &DbIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<u64, DbError>;
    fn remove(&mut self, index: &DbIndex) -> Result<(), DbError>;
    fn replace<T: Serialize>(&mut self, index: &DbIndex, value: &T) -> Result<u64, DbError>;
    fn replace_with_bytes(&mut self, index: &DbIndex, bytes: &[u8]) -> Result<u64, DbError>;
    fn resize_value(&mut self, index: &DbIndex, new_size: u64) -> Result<u64, DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn transaction(&mut self);
    fn value<T: Serialize>(&self, index: &DbIndex) -> Result<T, DbError>;
    fn value_as_bytes(&self, index: &DbIndex) -> Result<Vec<u8>, DbError>;
    fn value_as_bytes_at(&self, index: &DbIndex, offset: u64) -> Result<Vec<u8>, DbError>;
    fn value_as_bytes_at_size(
        &self,
        index: &DbIndex,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, DbError>;
    fn value_at<T: Serialize>(&self, index: &DbIndex, offset: u64) -> Result<T, DbError>;
    fn value_size(&self, index: &DbIndex) -> Result<u64, DbError>;
}
