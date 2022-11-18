pub mod file_storage;
pub mod storage_file;
pub mod storage_index;

mod storage_data;
mod storage_data_file;
mod storage_impl;
mod storage_record;
mod storage_records;
mod write_ahead_log;

use crate::db::db_error::DbError;
use crate::old_storage::storage_index::StorageIndex;
use crate::utilities::old_serialize::OldSerialize;
use crate::utilities::serialize::Serialize;
use crate::DbIndex;

pub trait OldStorage {
    fn commit(&mut self) -> Result<(), DbError>;
    fn insert<V: OldSerialize>(&mut self, value: &V) -> Result<StorageIndex, DbError>;
    fn insert_at<V: OldSerialize>(
        &mut self,
        index: &StorageIndex,
        offset: u64,
        value: &V,
    ) -> Result<u64, DbError>;
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
    fn value<V: OldSerialize>(&mut self, index: &StorageIndex) -> Result<V, DbError>;
    fn value_at<V: OldSerialize>(
        &mut self,
        index: &StorageIndex,
        offset: u64,
    ) -> Result<V, DbError>;
    fn value_size(&self, index: &StorageIndex) -> Result<u64, DbError>;
}

pub trait Storage {
    fn commit(&mut self) -> Result<(), DbError>;
    fn insert<T: Serialize>(&mut self, value: &T) -> Result<DbIndex, DbError>;
    fn insert_at<T: Serialize>(
        &mut self,
        index: &DbIndex,
        offset: usize,
        value: &T,
    ) -> Result<usize, DbError>;
    fn move_at<T: Serialize>(
        &mut self,
        index: &DbIndex,
        offset: usize,
        value: &T,
    ) -> Result<(), DbError>;
    fn replace<T: Serialize>(&mut self, index: &DbIndex, value: &T) -> Result<usize, DbError>;
    fn value<T: Serialize>(&self, index: &DbIndex) -> Result<T, DbError>;
    fn value_at<T: Serialize>(&self, index: &DbIndex, offset: usize) -> Result<T, DbError>;
    fn resize_value(&mut self, index: &DbIndex, new_size: u64) -> Result<(), DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn size(&mut self) -> Result<u64, DbError>;
    fn transaction(&mut self);
    fn value_size(&self, index: &DbIndex) -> Result<usize, DbError>;
}
