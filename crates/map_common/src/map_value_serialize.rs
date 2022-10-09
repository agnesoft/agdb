use crate::map_value::MapValue;
use crate::map_value_state::MapValueState;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;

impl<K, T> Serialize for MapValue<K, T>
where
    K: Clone + Default + Serialize,
    T: Clone + Default + Serialize,
{
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            state: MapValueState::deserialize(bytes)?,
            key: K::deserialize(&bytes[(MapValueState::serialized_size() as usize)..])?,
            value: T::deserialize(
                &bytes[((MapValueState::serialized_size() + K::serialized_size()) as usize)..],
            )?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.reserve(Self::serialized_size() as usize);
        data.extend(self.state.serialize());
        data.extend(self.key.serialize());
        data.extend(self.value.serialize());

        data
    }

    fn serialized_size() -> u64 {
        MapValueState::serialized_size() + K::serialized_size() + T::serialized_size()
    }
}
