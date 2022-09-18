use super::serialize::Serialize;
use super::storage_hash_map_meta_value::MetaValue;
use crate::DbError;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct StorageHashMapKeyValue<K: Serialize, T: Serialize> {
    key: K,
    value: T,
    meta_value: MetaValue,
}

impl<K: Serialize, T: Serialize> StorageHashMapKeyValue<K, T> {
    pub(crate) fn serialized_size() -> u64 {
        Self::meta_value_offset() + MetaValue::serialized_size()
    }

    pub(crate) fn meta_value_offset() -> u64 {
        std::mem::size_of::<K>() as u64 + std::mem::size_of::<T>() as u64
    }
}

impl<K: Serialize, T: Serialize> Serialize for StorageHashMapKeyValue<K, T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            key: K::deserialize(&bytes[0..])?,
            value: T::deserialize(&bytes[std::mem::size_of::<K>()..])?,
            meta_value: MetaValue::deserialize(
                &bytes[(std::mem::size_of::<K>() + std::mem::size_of::<T>())..],
            )?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.reserve(std::mem::size_of::<K>() + std::mem::size_of::<T>() + 1);
        data.append(&mut self.key.serialize());
        data.append(&mut self.value.serialize());
        data.append(&mut self.meta_value.serialize());

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let key_value = StorageHashMapKeyValue::<i64, i64>::default();

        format!("{:?}", key_value);
    }

    #[test]
    fn derived_from_default() {
        let key_value = StorageHashMapKeyValue::<i64, i64>::default();

        assert_eq!(
            key_value,
            StorageHashMapKeyValue::<i64, i64> {
                key: 0,
                value: 0,
                meta_value: MetaValue::Empty
            }
        )
    }

    #[test]
    fn i64_i64() {
        let key_value = StorageHashMapKeyValue {
            key: 1_i64,
            value: 10_i64,
            meta_value: MetaValue::Valid,
        };
        let bytes = key_value.serialize();
        let other = StorageHashMapKeyValue::deserialize(&bytes);

        assert_eq!(other, Ok(key_value));
    }

    #[test]
    fn out_of_bounds() {
        let bytes = vec![0_u8; 16];

        assert_eq!(
            StorageHashMapKeyValue::<i64, i64>::deserialize(&bytes)
                .unwrap_err()
                .description,
            "value out of bounds"
        );
    }

    #[test]
    fn serialized_size() {
        assert_eq!(StorageHashMapKeyValue::<i64, i64>::serialized_size(), 17);
    }
}
