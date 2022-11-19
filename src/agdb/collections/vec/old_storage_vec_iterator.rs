use crate::collections::old_storage_vec::OldStorageVec;
use crate::old_storage::OldStorage;
use crate::utilities::old_serialize::OldSerialize;
use std::marker::PhantomData;

pub struct OldStorageVecIterator<'a, T, Data>
where
    T: OldSerialize,
    Data: OldStorage,
{
    pub(crate) index: u64,
    pub(crate) vec: &'a OldStorageVec<T, Data>,
    pub(crate) phantom_data: PhantomData<T>,
}

impl<'a, T, Data> Iterator for OldStorageVecIterator<'a, T, Data>
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
