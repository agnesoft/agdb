use super::hash_map_meta_value::HashMapMetaValue;
use crate::DbError;
use serialize::Serialize;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HashMapKeyValue<K, T>
where
    K: Clone + Default + Serialize,
    T: Clone + Default + Serialize,
{
    pub(super) meta_value: HashMapMetaValue,
    pub(super) key: K,
    pub(super) value: T,
}

impl<K, T> Serialize for HashMapKeyValue<K, T>
where
    K: Clone + Default + Serialize,
    T: Clone + Default + Serialize,
{
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            meta_value: HashMapMetaValue::deserialize(bytes)?,
            key: K::deserialize(&bytes[(HashMapMetaValue::serialized_size() as usize)..])?,
            value: T::deserialize(
                &bytes[((HashMapMetaValue::serialized_size() + K::serialized_size()) as usize)..],
            )?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.reserve(Self::serialized_size() as usize);
        data.extend(self.meta_value.serialize());
        data.extend(self.key.serialize());
        data.extend(self.value.serialize());

        data
    }

    fn serialized_size() -> u64 {
        HashMapMetaValue::serialized_size() + K::serialized_size() + T::serialized_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let key_value = HashMapKeyValue::<i64, i64>::default();

        format!("{:?}", key_value);
    }

    #[test]
    fn derived_from_default() {
        let key_value = HashMapKeyValue::<i64, i64>::default();

        assert_eq!(
            key_value,
            HashMapKeyValue::<i64, i64> {
                meta_value: HashMapMetaValue::Empty,
                key: 0,
                value: 0,
            }
        )
    }

    #[test]
    fn i64_i64() {
        let key_value = HashMapKeyValue {
            meta_value: HashMapMetaValue::Valid,
            key: 1_i64,
            value: 10_i64,
        };
        let bytes = key_value.serialize();
        let other = HashMapKeyValue::deserialize(&bytes);

        assert_eq!(other, Ok(key_value));
    }

    #[test]
    fn out_of_bounds() {
        let bytes = vec![0_u8; 16];

        assert_eq!(
            HashMapKeyValue::<i64, i64>::deserialize(&bytes)
                .unwrap_err()
                .description,
            "i64 deserialization error: out of bounds"
        );
    }

    #[test]
    fn serialized_size() {
        assert_eq!(HashMapKeyValue::<i64, i64>::serialized_size(), 17);
    }
}
