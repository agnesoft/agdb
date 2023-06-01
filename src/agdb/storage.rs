pub mod file_storage;

mod file_record;
mod file_records;
mod write_ahead_log;

use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct StorageIndex(pub u64);

impl From<u64> for StorageIndex {
    fn from(index: u64) -> Self {
        Self(index)
    }
}

impl Serialize for StorageIndex {
    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self(u64::deserialize(bytes)?))
    }

    fn serialized_size(&self) -> u64 {
        self.0.serialized_size()
    }
}

impl SerializeStatic for StorageIndex {}

pub trait Storage {
    fn commit(&mut self, id: u64) -> Result<(), DbError>;
    fn insert<T: Serialize>(&mut self, value: &T) -> Result<StorageIndex, DbError>;
    fn insert_at<T: Serialize>(
        &mut self,
        index: StorageIndex,
        offset: u64,
        value: &T,
    ) -> Result<(), DbError>;
    fn insert_bytes(&mut self, bytes: &[u8]) -> Result<StorageIndex, DbError>;
    fn insert_bytes_at(
        &mut self,
        index: StorageIndex,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), DbError>;
    fn len(&self) -> Result<u64, DbError>;
    fn move_at(
        &mut self,
        index: StorageIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError>;
    fn remove(&mut self, index: StorageIndex) -> Result<(), DbError>;
    fn replace<T: Serialize>(&mut self, index: StorageIndex, value: &T) -> Result<(), DbError>;
    fn replace_with_bytes(&mut self, index: StorageIndex, bytes: &[u8]) -> Result<(), DbError>;
    fn resize_value(&mut self, index: StorageIndex, new_size: u64) -> Result<(), DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn transaction(&mut self) -> u64;
    fn value<T: Serialize>(&self, index: StorageIndex) -> Result<T, DbError>;
    fn value_as_bytes(&self, index: StorageIndex) -> Result<Vec<u8>, DbError>;
    fn value_as_bytes_at(&self, index: StorageIndex, offset: u64) -> Result<Vec<u8>, DbError>;
    fn value_as_bytes_at_size(
        &self,
        index: StorageIndex,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, DbError>;
    fn value_at<T: Serialize>(&self, index: StorageIndex, offset: u64) -> Result<T, DbError>;
    fn value_size(&self, index: StorageIndex) -> Result<u64, DbError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn derived_from_clone() {
        let index = StorageIndex::default();
        let other = index.clone();
        assert_eq!(index, other);
    }

    #[test]
    fn derived_from_debug() {
        format!("{:?}", StorageIndex::default());
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(
            StorageIndex::default().cmp(&StorageIndex::default()),
            Ordering::Equal
        );
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut indexes = vec![
            StorageIndex::default(),
            StorageIndex::from(100_u64),
            StorageIndex::from(u64::MAX),
            StorageIndex::from(1_u64),
        ];
        indexes.sort();
        assert_eq!(
            indexes,
            vec![
                StorageIndex::default(),
                StorageIndex::from(1_u64),
                StorageIndex::from(100_u64),
                StorageIndex::from(u64::MAX),
            ]
        )
    }

    #[test]
    fn serialize() {
        let index = StorageIndex::from(1_u64);
        let bytes = index.serialize();
        let other = StorageIndex::deserialize(&bytes).unwrap();
        assert_eq!(index, other);
    }
}
