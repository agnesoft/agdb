use crate::storage::storage_value::StorageValue;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize_static::SerializeStatic;
use crate::DbError;
use std::mem::size_of;

#[derive(Debug, Default, Eq, PartialEq)]
pub enum MapValueState {
    #[default]
    Empty,
    Deleted,
    Valid,
}

impl Serialize for MapValueState {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        match bytes.first() {
            Some(0) => Ok(MapValueState::Empty),
            Some(1) => Ok(MapValueState::Valid),
            Some(2) => Ok(MapValueState::Deleted),
            _ => Err(DbError::from(
                "MapValueState deserialization error: unknown value",
            )),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        match self {
            MapValueState::Empty => vec![0_u8],
            MapValueState::Deleted => vec![2_u8],
            MapValueState::Valid => vec![1_u8],
        }
    }

    fn serialized_size(&self) -> u64 {
        size_of::<u8>() as u64
    }
}

impl SerializeStatic for MapValueState {
    fn static_serialized_size() -> u64 {
        size_of::<u8>() as u64
    }
}

impl StorageValue for MapValueState {
    fn store<S: crate::storage::Storage>(&self, _storage: &mut S) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load<S: crate::storage::Storage>(_storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        Self::deserialize(bytes)
    }

    fn remove<S: crate::storage::Storage>(_storage: &mut S, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        Self::static_serialized_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_deserialize() {
        assert_eq!(
            MapValueState::deserialize(&Vec::<u8>::new()).err().unwrap(),
            DbError::from("MapValueState deserialization error: unknown value")
        );
    }

    #[test]
    fn derived_from_debug() {
        let value = MapValueState::Deleted;
        format!("{:?}", value);
    }

    #[test]
    fn derived_from_default() {
        assert_eq!(MapValueState::default(), MapValueState::Empty);
    }

    #[test]
    fn serialized_size() {
        assert_eq!(MapValueState::default().serialized_size(), 1);
    }
}
