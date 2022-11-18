use crate::collections::storage_vec::StorageVec;
use crate::storage::OldStorage;
use crate::utilities::old_serialize::OldSerialize;
use std::marker::PhantomData;

pub struct StorageVecIterator<'a, T, Data>
where
    T: OldSerialize,
    Data: OldStorage,
{
    pub(crate) index: u64,
    pub(crate) vec: &'a StorageVec<T, Data>,
    pub(crate) phantom_data: PhantomData<T>,
}

impl<'a, T, Data> Iterator for StorageVecIterator<'a, T, Data>
where
    T: OldSerialize,
    Data: OldStorage,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.value(self.index).ok();
        self.index += 1;

        value
    }
}
