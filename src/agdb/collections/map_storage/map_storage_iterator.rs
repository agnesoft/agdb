use super::MapStorage;
use crate::collections::map::map_value_state::MapValueState;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct MapStorageIterator<'a, K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash + StorageValue + Display,
    T: Default + StorageValue,
    Data: Storage,
{
    pub pos: u64,
    pub map: &'a MapStorage<K, T, Data>,
    pub phantom_data: PhantomData<(K, T)>,
}

impl<'a, K, T, Data> Iterator for MapStorageIterator<'a, K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash + StorageValue + Display,
    T: Default + StorageValue,
    Data: Storage,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos != self.map.capacity() {
            let current_pos = self.pos;
            self.pos += 1;

            if self.map.states.value(current_pos).unwrap_or_default() == MapValueState::Valid {
                let key = self.map.keys.value(current_pos).unwrap_or_default();
                let value = self.map.values.value(current_pos).unwrap_or_default();

                return Some((key, value));
            }
        }

        None
    }
}
