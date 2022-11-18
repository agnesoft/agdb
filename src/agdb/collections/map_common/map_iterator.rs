use super::map_data::MapData;
use super::map_value_state::MapValueState;
use crate::utilities::old_serialize::OldSerialize;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct MapIterator<'a, K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + OldSerialize,
    T: Clone + Default + OldSerialize,
    Data: MapData<K, T>,
{
    pub(crate) pos: u64,
    pub(crate) data: &'a Data,
    pub(crate) phantom_data: PhantomData<(K, T)>,
}

impl<'a, K, T, Data> Iterator for MapIterator<'a, K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + OldSerialize,
    T: Clone + Default + OldSerialize,
    Data: MapData<K, T>,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.data.capacity() {
            let value = self.data.record(self.pos).unwrap_or_default();

            self.pos += 1;

            if value.state == MapValueState::Valid {
                return Some((value.key, value.value));
            }
        }

        None
    }
}
