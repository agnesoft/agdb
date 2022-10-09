use crate::storage_vec::StorageVec;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use std::marker::PhantomData;

pub struct VecIterator<'a, T, Data>
where
    T: Serialize,
    Data: Storage,
{
    pub(crate) index: u64,
    pub(crate) vec: &'a StorageVec<T, Data>,
    pub(crate) phantom_data: PhantomData<T>,
}

impl<'a, T, Data> Iterator for VecIterator<'a, T, Data>
where
    T: Serialize,
    Data: Storage,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.value(self.index).ok();
        self.index += 1;

        value
    }
}
