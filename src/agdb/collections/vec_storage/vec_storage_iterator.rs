use crate::collections::vec_storage::VecStorage;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use std::marker::PhantomData;

pub struct VecStorageIterator<'a, T, Data>
where
    T: StorageValue,
    Data: Storage,
{
    pub(crate) index: u64,
    pub(crate) vec: &'a VecStorage<T, Data>,
    pub(crate) phantom_data: PhantomData<T>,
}

impl<'a, T, Data> Iterator for VecStorageIterator<'a, T, Data>
where
    T: StorageValue,
    Data: Storage,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.value(self.index).ok();
        self.index += 1;

        value
    }
}
