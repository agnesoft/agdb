use crate::collections::vec_storage::VecStorage;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use std::marker::PhantomData;

pub struct VecStorageIterator<'a, T, Data>
where
    T: Serialize,
    Data: Storage,
{
    pub(crate) index: u64,
    pub(crate) vec: &'a VecStorage<T, Data>,
    pub(crate) phantom_data: PhantomData<T>,
}

impl<'a, T, Data> Iterator for VecStorageIterator<'a, T, Data>
where
    T: Serialize,
    Data: Storage,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // let value = self.vec.value(self.index).ok();
        // self.index += 1;

        // value
    }
}
