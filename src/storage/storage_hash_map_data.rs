use super::serialize::Serialize;
use super::storage_hash_map_key_value::StorageHashMapKeyValue;

#[derive(Debug, PartialEq)]
pub(crate) struct StorageHashMapData<K: Serialize, T: Serialize> {
    pub(crate) data: Vec<StorageHashMapKeyValue<K, T>>,
    pub(crate) size: u64,
}

impl<K: Serialize, T: Serialize> Serialize for StorageHashMapData<K, T> {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        let size = u64::deserialize(bytes)?;

        const SIZE_OFFSET: usize = std::mem::size_of::<usize>();
        let value_offset = StorageHashMapKeyValue::<K, T>::serialized_size() as usize;
        let data_size = (bytes.len() - SIZE_OFFSET) / value_offset;
        let mut data = Vec::<StorageHashMapKeyValue<K, T>>::new();
        data.reserve(data_size);

        for i in 0..data_size {
            let offset = SIZE_OFFSET + value_offset * i;
            data.push(StorageHashMapKeyValue::<K, T>::deserialize(
                &bytes[offset..],
            )?);
        }

        Ok(StorageHashMapData { data, size })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        let byte_size = std::mem::size_of::<u64>()
            + self.data.len() * StorageHashMapKeyValue::<K, T>::serialized_size() as usize;

        bytes.reserve(byte_size);
        bytes.extend(self.size.serialize());

        for record in &self.data {
            bytes.extend(record.serialize());
        }

        bytes
    }

    fn serialized_size() -> u64 {
        0
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
        let other = StorageHashMapData::<i64, i64>::deserialize(&bytes).unwrap();

        assert_eq!(data, other);
    }
}
