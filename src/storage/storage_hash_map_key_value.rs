use super::serialize::Serialize;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct StorageHashMapKeyValue<K: Serialize, T: Serialize> {
    key: K,
    value: T,
}

impl<K: Serialize, T: Serialize> Serialize for StorageHashMapKeyValue<K, T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        Ok(Self {
            key: K::deserialize(&bytes[0..])?,
            value: T::deserialize(&bytes[std::mem::size_of::<K>()..])?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.reserve(std::mem::size_of::<K>() + std::mem::size_of::<T>());
        data.append(&mut self.key.serialize());
        data.append(&mut self.value.serialize());

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64_i64() {
        let key_value = StorageHashMapKeyValue {
            key: 1_i64,
            value: 10_i64,
        };
        let bytes = key_value.serialize();
        let other = StorageHashMapKeyValue::deserialize(&bytes);

        assert_eq!(other, Ok(key_value));
    }
}
