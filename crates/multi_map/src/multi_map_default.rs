use crate::MultiMap;
use agdb_serialize::Serialize;
use agdb_utilities::StableHash;
use std::hash::Hash;

impl<K, T> Default for MultiMap<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}
