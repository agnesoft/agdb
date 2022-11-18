use super::dictionary_index::DictionaryIndex;
use super::dictionary_value::DictionaryValue;
use crate::db::db_error::DbError;
use crate::utilities::serialize::OldSerialize;
use crate::utilities::stable_hash::StableHash;

pub trait DictionaryData<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + OldSerialize,
{
    fn capacity(&self) -> u64;
    fn commit(&mut self) -> Result<(), DbError>;
    fn indexes(&self, hash: u64) -> Result<Vec<DictionaryIndex>, DbError>;
    fn insert(&mut self, hash: u64, index: &DictionaryIndex) -> Result<(), DbError>;
    fn hash(&self, index: &DictionaryIndex) -> Result<u64, DbError>;
    fn meta(&self, index: &DictionaryIndex) -> Result<i64, DbError>;
    fn remove(&mut self, hash: u64, index: &DictionaryIndex) -> Result<(), DbError>;
    fn set_hash(&mut self, index: &DictionaryIndex, hash: u64) -> Result<(), DbError>;
    fn set_meta(&mut self, index: &DictionaryIndex, meta: i64) -> Result<(), DbError>;
    fn set_value(
        &mut self,
        index: &DictionaryIndex,
        value: DictionaryValue<T>,
    ) -> Result<(), DbError>;
    fn transaction(&mut self);
    fn value(&self, index: &DictionaryIndex) -> Result<DictionaryValue<T>, DbError>;
}
