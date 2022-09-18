use super::serialize::Serialize;
use super::storage_hash_map_key_value::StorageHashMapKeyValue;

#[derive(Debug, PartialEq)]
pub(crate) struct StorageHashMapData<K: Serialize, T: Serialize> {
    pub(crate) data: Vec<StorageHashMapKeyValue<K, T>>,
    pub(crate) size: u64,
}

impl<K: Serialize, T: Serialize> Serialize for StorageHashMapData<K, T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        
    }

    fn serialize(&self) -> Vec<u8> {
        let bytes = Vec<u8>::new();
        bytes.reserve()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let data = StorageHashMapData::<i64, i64> {
            data: Default::default(),
            size: 0,
        };

        format!("{:?}", data);
    }

    #[test]
    fn serialization() {
        let data = StorageHashMapData::<i64, i64> {
            data: vec![
                StorageHashMapKeyValue::<i64, i64>::default(),
                StorageHashMapKeyValue::<i64, i64>::default(),
                StorageHashMapKeyValue::<i64, i64>::default(),
            ],
            size: 2,
        };
        let bytes = data.serialize();
        let other = StorageHashMapData::<i64, i64>::deserialize()?;

        assert_eq!(data, other);
    }
}
