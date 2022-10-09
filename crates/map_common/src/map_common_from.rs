use crate::map_common::MapCommon;
use crate::map_data::MapData;
use agdb_serialize::Serialize;
use agdb_utilities::StableHash;
use std::hash::Hash;
use std::marker::PhantomData;

impl<K, T, Data> From<Data> for MapCommon<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: MapData<K, T>,
{
    fn from(data: Data) -> Self {
        Self {
            data,
            phantom_data: PhantomData,
        }
    }
}
