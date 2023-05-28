use crate::db::db_error::DbError;
use crate::storage::storage_value::StorageValue;
use crate::utilities::serialize::{Serialize, SerializeStatic};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
        Self::serialized_size_static()
    }
}

impl SerializeStatic for MapValueState {
    fn serialized_size_static() -> u64 {
        1
    }
}

impl StorageValue for MapValueState {
    fn storage_len() -> u64 {
        Self::serialized_size_static()
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
        format!("{value:?}");
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
