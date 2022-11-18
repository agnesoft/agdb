use super::storage_index::StorageIndex;
use crate::utilities::partial_serialize::PartialSerialize;
use crate::DbError;

pub trait PartialStorage {
    fn commit(&mut self) -> Result<(), DbError>;
    fn insert<T: PartialSerialize>(&mut self, value: &T) -> Result<StorageIndex, DbError>;
    fn insert_at<T: PartialSerialize>(
        &mut self,
        index: &StorageIndex,
        offset: usize,
        value: &T,
    ) -> Result<usize, DbError>;
    fn move_at<T: PartialSerialize>(
        &mut self,
        index: &StorageIndex,
        offset: usize,
        value: &T,
    ) -> Result<(), DbError>;
    fn replace<T: PartialSerialize>(
        &mut self,
        index: &StorageIndex,
        value: &T,
    ) -> Result<usize, DbError>;
    fn value<T: PartialSerialize>(&self, index: &StorageIndex) -> Result<T, DbError>;
    fn value_at<T: PartialSerialize>(
        &self,
        index: &StorageIndex,
        offset: usize,
    ) -> Result<T, DbError>;
    fn resize_value(&mut self, index: &StorageIndex, new_size: u64) -> Result<(), DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn size(&mut self) -> Result<u64, DbError>;
    fn transaction(&mut self);
    fn value_size(&self, index: &StorageIndex) -> Result<usize, DbError>;
}
