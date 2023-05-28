use crate::db::db_error::DbError;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;

pub struct MapDataStorageIndex {
    pub len: u64,
    pub states_index: StorageIndex,
    pub keys_index: StorageIndex,
    pub values_index: StorageIndex,
}

impl SerializeStatic for MapDataStorageIndex {
    fn serialized_size_static() -> u64 {
        u64::serialized_size_static() + StorageIndex::serialized_size_static() * 3
    }
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

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() < Self::serialized_size_static() as usize {
            return Err(DbError::from(
                "MapDataStorageIndex deserialization error: not enough data",
            ));
        }

        Ok(MapDataStorageIndex {
            len: u64::deserialize(bytes)?,
            states_index: StorageIndex::deserialize(
                &bytes[u64::serialized_size_static() as usize..],
            )?,
            keys_index: StorageIndex::deserialize(
                &bytes[(u64::serialized_size_static() + StorageIndex::serialized_size_static())
                    as usize..],
            )?,
            values_index: StorageIndex::deserialize(
                &bytes[(u64::serialized_size_static() + StorageIndex::serialized_size_static() * 2)
                    as usize..],
            )?,
        })
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

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
