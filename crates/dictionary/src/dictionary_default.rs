use crate::Dictionary;
use agdb_serialize::Serialize;
use agdb_utilities::StableHash;

impl<T> Default for Dictionary<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}
