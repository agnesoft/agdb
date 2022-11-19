use super::vec_fixed_sized::VecFixedSized;
use crate::collections::vec_storage::VecStorage;
use crate::storage::Storage;
use crate::utilities::serialize::SerializeFixedSized;
use std::marker::PhantomData;

pub struct VecFixedSizedIterator<'a, T, Data>
where
    T: SerializeFixedSized,
    Data: Storage,
{
    pub(crate) index: u64,
    pub(crate) vec: &'a VecStorage<T, Data>,
    pub(crate) phantom_data: PhantomData<T>,
}

impl<'a, T, Data> Iterator for VecFixedSizedIterator<'a, T, Data>
where
    T: SerializeFixedSized,
    Data: Storage,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.value(self.index).ok();
        self.index += 1;

        value
    }
}
