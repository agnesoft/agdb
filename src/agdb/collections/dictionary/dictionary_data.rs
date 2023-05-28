use crate::collections::vec::VecValue;
use crate::db::db_error::DbError;
use crate::utilities::stable_hash::StableHash;

pub trait DictionaryData<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + VecValue,
{
    fn capacity(&self) -> u64;
    fn commit(&mut self, id: u64) -> Result<(), DbError>;
    fn count(&self, index: u64) -> Result<u64, DbError>;
    fn hash(&self, index: u64) -> Result<u64, DbError>;
    fn indexes(&self, hash: u64) -> Result<Vec<u64>, DbError>;
    fn insert(&mut self, hash: u64, index: u64) -> Result<(), DbError>;
    fn remove(&mut self, hash: u64, index: u64) -> Result<(), DbError>;
    fn set_capacity(&mut self, capacity: u64) -> Result<(), DbError>;
    fn set_count(&mut self, index: u64, count: u64) -> Result<(), DbError>;
    fn set_hash(&mut self, index: u64, hash: u64) -> Result<(), DbError>;
    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError>;
    fn transaction(&mut self) -> u64;
    fn value(&self, index: u64) -> Result<T, DbError>;
}
