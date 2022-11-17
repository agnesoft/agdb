use super::map_value_state::MapValueState;
use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;

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

impl<K, T> Serialize for MapValue<K, T>
where
    K: Clone + Default + Serialize,
    T: Clone + Default + Serialize,
{
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            state: MapValueState::deserialize(bytes)?,
            key: K::deserialize(&bytes[(MapValueState::fixed_size() as usize)..])?,
            value: T::deserialize(
                &bytes[((MapValueState::fixed_size() + K::fixed_size()) as usize)..],
            )?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.reserve(Self::fixed_size() as usize);
        data.extend(self.state.serialize());
        data.extend(self.key.serialize());
        data.extend(self.value.serialize());

        data
    }

    fn serialized_size(&self) -> u64 {
        todo!()
    }

    fn fixed_size() -> u64 {
        MapValueState::fixed_size() + K::fixed_size() + T::fixed_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let key_value = MapValue::<i64, i64>::default();
        format!("{:?}", key_value);
    }
    #[test]
    fn derived_from_default() {
        let key_value = MapValue::<i64, i64>::default();
        assert_eq!(
            key_value,
            MapValue::<i64, i64> {
                state: MapValueState::Empty,
                key: 0,
                value: 0,
            }
        )
    }
    #[test]
    fn i64_i64() {
        let key_value = MapValue {
            state: MapValueState::Valid,
            key: 1_i64,
            value: 10_i64,
        };
        let bytes = key_value.serialize();
        let other = MapValue::deserialize(&bytes);
        assert_eq!(other, Ok(key_value));
    }
    #[test]
    fn out_of_bounds() {
        let bytes = vec![0_u8; 16];
        assert_eq!(
            MapValue::<i64, i64>::deserialize(&bytes)
                .unwrap_err()
                .description,
            "i64 deserialization error: out of bounds"
        );
    }
    #[test]
    fn serialized_size() {
        assert_eq!(MapValue::<i64, i64>::fixed_size(), 17);
    }
}
