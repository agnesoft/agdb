use crate::storage::storage_index::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize_static::SerializeStatic;
use crate::DbError;

pub struct MapDataStorageIndex {
    pub len: u64,
    pub states_index: StorageIndex,
    pub keys_index: StorageIndex,
    pub values_index: StorageIndex,
}

impl Serialize for MapDataStorageIndex {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size() as usize);
        bytes.extend(self.len.serialize());
        bytes.extend(self.states_index.serialize());
        bytes.extend(self.keys_index.serialize());
        bytes.extend(self.values_index.serialize());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        if bytes.len() < Self::static_serialized_size() as usize {
            return Err(DbError::from(
                "MapDataStorageIndex deserialization error: not enough data",
            ));
        }

        Ok(MapDataStorageIndex {
            len: u64::deserialize(bytes)?,
            states_index: StorageIndex::deserialize(
                &bytes[u64::static_serialized_size() as usize..],
            )?,
            keys_index: StorageIndex::deserialize(
                &bytes[(u64::static_serialized_size() + StorageIndex::static_serialized_size())
                    as usize..],
            )?,
            values_index: StorageIndex::deserialize(
                &bytes[(u64::static_serialized_size() + StorageIndex::static_serialized_size() * 2)
                    as usize..],
            )?,
        })
    }

    fn serialized_size(&self) -> u64 {
        Self::static_serialized_size()
    }
}

impl SerializeStatic for MapDataStorageIndex {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_deserialize() {
        assert_eq!(
            MapDataStorageIndex::deserialize(&Vec::<u8>::new())
                .err()
                .unwrap(),
            DbError::from("MapDataStorageIndex deserialization error: not enough data")
        );
    }
}
