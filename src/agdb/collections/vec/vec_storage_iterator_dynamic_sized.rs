use crate::collections::vec_storage::VecDynamicSized;
use crate::collections::vec_storage::VecStorage;
use crate::storage::Storage;
use crate::utilities::serialize::SerializeDynamicSized;
use std::marker::PhantomData;

pub struct VecStorageIteratorDynamicSized<'a, T, Data>
where
    T: SerializeDynamicSized,
    Data: Storage,
{
    pub(crate) index: u64,
    pub(crate) vec: &'a VecStorage<T, Data>,
    pub(crate) phantom_data: PhantomData<T>,
}

impl<'a, T, Data> Iterator for VecStorageIteratorDynamicSized<'a, T, Data>
where
    T: SerializeDynamicSized,
    Data: Storage,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.value(self.index).ok();
        self.index += 1;

        value
    }
}
