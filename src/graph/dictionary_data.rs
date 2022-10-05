use super::dictionary_value::DictionaryValue;
use crate::storage::Serialize;
use crate::storage::StableHash;
use crate::DbError;

pub(crate) trait DictionaryData<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    fn capacity(&self) -> u64;
    fn commit(&mut self) -> Result<(), DbError>;
    fn indexes(&self, hash: u64) -> Result<Vec<i64>, DbError>;
    fn insert(&mut self, hash: u64, index: i64) -> Result<(), DbError>;
    fn hash(&self, index: i64) -> Result<u64, DbError>;
    fn meta(&self, index: i64) -> Result<i64, DbError>;
    fn remove(&mut self, hash: u64, index: i64) -> Result<(), DbError>;
    fn set_hash(&mut self, index: i64, hash: u64) -> Result<(), DbError>;
    fn set_meta(&mut self, index: i64, meta: i64) -> Result<(), DbError>;
    fn set_value(&mut self, index: i64, value: DictionaryValue<T>) -> Result<(), DbError>;
    fn transaction(&mut self);
    fn value(&self, index: i64) -> Result<DictionaryValue<T>, DbError>;
}
