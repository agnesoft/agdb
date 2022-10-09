use agdb_serialize::Serialize;
use agdb_utilities::StableHash;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct DictionaryValue<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    pub(crate) meta: i64,
    pub(crate) hash: u64,
    pub(crate) value: T,
}
