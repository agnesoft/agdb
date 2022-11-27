use super::MapStorage;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct MapStorageIterator<'a, K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash + StorageValue,
    T: Default + StorageValue,
    Data: Storage,
{
    pub pos: u64,
    pub map: &'a MapStorage<K, T, Data>,
    pub phantom_data: PhantomData<(K, T)>,
}

impl<'a, K, T, Data> Iterator for MapStorageIterator<'a, K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash + StorageValue,
    T: Default + StorageValue,
    Data: Storage,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
