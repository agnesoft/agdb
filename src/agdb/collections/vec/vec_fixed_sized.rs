use super::vec_fixed_sized_iterator::VecFixedSizedIterator;
use crate::storage::Storage;
use crate::utilities::serialize::SerializeFixedSized;
use crate::DbError;
use crate::DbIndex;
use std::cell::RefCell;
use std::rc::Rc;

pub trait VecFixedSized<T, Data>: Sized
where
    T: SerializeFixedSized,
    Data: Storage,
{
    fn from_storage(storage: Rc<RefCell<Data>>, index: &DbIndex) -> Result<Self, DbError>;
    fn iter(&self) -> VecFixedSizedIterator<T, Data>;
    fn push(&mut self, value: &T) -> Result<(), DbError>;
    fn remove(&mut self, index: u64) -> Result<(), DbError>;
    fn reserve(&mut self, capacity: u64) -> Result<(), DbError>;
    fn resize(&mut self, size: u64, value: &T) -> Result<(), DbError>;
    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn to_vec(&self) -> Result<Vec<T>, DbError>;
    fn value(&self, index: u64) -> Result<T, DbError>;
}
