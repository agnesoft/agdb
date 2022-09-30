use crate::DbError;

use super::serialize::Serialize;

#[derive(Clone, Default, Debug, PartialEq)]
pub(crate) enum HashMapMetaValue {
    #[default]
    Empty,
    Deleted,
    Valid,
}

impl Serialize for HashMapMetaValue {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        match bytes.first() {
            Some(0) => Ok(HashMapMetaValue::Empty),
            Some(1) => Ok(HashMapMetaValue::Valid),
            Some(2) => Ok(HashMapMetaValue::Deleted),
            _ => Err(DbError::from("value out of bounds")),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        match self {
            HashMapMetaValue::Empty => vec![0_u8],
            HashMapMetaValue::Deleted => vec![2_u8],
            HashMapMetaValue::Valid => vec![1_u8],
        }
    }

    fn serialized_size() -> u64 {
        std::mem::size_of::<u8>() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_default() {
        assert_eq!(HashMapMetaValue::default(), HashMapMetaValue::Empty);
    }

    #[test]
    fn derived_from_debug() {
        let value = HashMapMetaValue::Deleted;

        format!("{:?}", value);
    }

    #[test]
    fn serialize() {
        let data = vec![
            HashMapMetaValue::Valid,
            HashMapMetaValue::Empty,
            HashMapMetaValue::Deleted,
        ];
        let bytes = data.serialize();
        let other = Vec::<HashMapMetaValue>::deserialize(&bytes).unwrap();

        assert_eq!(data, other);
    }

    #[test]
    fn serialized_size() {
        assert_eq!(HashMapMetaValue::serialized_size(), 1);
    }
}
