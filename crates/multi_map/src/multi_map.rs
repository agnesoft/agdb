use crate::map_data_memory::MapDataMemory;
use crate::multi_map_impl::MultiMapImpl;
use agdb_map_common::MapCommon;
use agdb_serialize::Serialize;
use agdb_utilities::StableHash;
use std::hash::Hash;

pub type MultiMap<K, T> = MultiMapImpl<K, T, MapDataMemory<K, T>>;

impl<K, T> MultiMap<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
{
    pub fn new() -> MultiMap<K, T> {
        MultiMap::<K, T> {
            map_common: MapCommon::<K, T, MapDataMemory<K, T>>::from(
                MapDataMemory::<K, T>::default(),
            ),
        }
    }
}
