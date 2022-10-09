use super::map_data::MapData;
use super::map_value_state::MapValueState;
use agdb_serialize::Serialize;
use agdb_utilities::StableHash;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct MapIterator<'a, K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: MapData<K, T>,
{
    pub(super) pos: u64,
    pub(super) data: &'a Data,
    pub(super) phantom_data: PhantomData<(K, T)>,
}

impl<'a, K, T, Data> Iterator for MapIterator<'a, K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
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
