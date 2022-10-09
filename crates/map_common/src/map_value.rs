use super::map_value_state::MapValueState;
use agdb_serialize::Serialize;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MapValue<K, T>
where
    K: Clone + Default + Serialize,
    T: Clone + Default + Serialize,
{
    pub state: MapValueState,
    pub key: K,
    pub value: T,
}
