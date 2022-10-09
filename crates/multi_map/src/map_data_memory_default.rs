use crate::map_data_memory::MapDataMemory;
use agdb_map_common::MapValue;
use agdb_serialize::Serialize;
use agdb_utilities::StableHash;
use std::hash::Hash;

impl<K, T> Default for MapDataMemory<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
{
    fn default() -> Self {
        Self {
            data: vec![MapValue::<K, T>::default()],
            count: 0,
        }
    }
}
