use super::StorageVec;
use serialize::Serialize;
use storage::StorageData;

pub(crate) struct VecIterator<'a, T, Data>
where
    T: Serialize,
    Data: StorageData,
{
    pub(super) index: u64,
    pub(super) vec: &'a StorageVec<T, Data>,
    pub(super) phantom_data: std::marker::PhantomData<T>,
}

impl<'a, T, Data> Iterator for VecIterator<'a, T, Data>
where
    T: Serialize,
    Data: StorageData,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.value(self.index).ok();
        self.index += 1;

        value
    }
}
