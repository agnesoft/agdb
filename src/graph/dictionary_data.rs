use crate::storage::Serialize;
use crate::storage::StableHash;
use crate::DbError;

pub(crate) trait DictionaryData<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    fn commit(&mut self) -> Result<(), DbError>;
    fn transaction(&mut self);
}
