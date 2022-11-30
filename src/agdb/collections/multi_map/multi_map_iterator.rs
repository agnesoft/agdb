use crate::collections::map::map_data::MapData;
use crate::collections::map::map_value_state::MapValueState;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct MultiMapIterator<'a, K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    pub pos: u64,
    pub data: &'a Data,
    pub phantom_data: PhantomData<(K, T)>,
}

impl<'a, K, T, Data> Iterator for MultiMapIterator<'a, K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos != self.data.capacity() {
            let current_pos = self.pos;
            self.pos += 1;

            if self.data.state(current_pos).unwrap_or_default() == MapValueState::Valid {
                let key = self.data.key(current_pos).unwrap_or_default();
                let value = self.data.value(current_pos).unwrap_or_default();

                return Some((key, value));
            }
        }

        None
    }
}
