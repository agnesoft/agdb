use crate::DbError;

use super::serialize::Serialize;

#[allow(dead_code)]
#[derive(Default, Debug, PartialEq)]
pub(crate) enum MetaValue {
    #[default]
    Empty,
    Deleted,
    Valid,
}

impl MetaValue {
    pub(crate) fn serialized_size() -> u64 {
        std::mem::size_of::<u8>() as u64
    }
}

impl Serialize for MetaValue {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        match bytes.first() {
            Some(0) => Ok(MetaValue::Empty),
            Some(1) => Ok(MetaValue::Valid),
            Some(2) => Ok(MetaValue::Deleted),
            _ => Err(DbError::from("value out of bounds")),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        match self {
            MetaValue::Empty => vec![0_u8],
            MetaValue::Deleted => vec![2_u8],
            MetaValue::Valid => vec![1_u8],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_default() {
        assert_eq!(MetaValue::default(), MetaValue::Empty);
    }

    #[test]
    fn derived_from_debug() {
        let value = MetaValue::Deleted;

        format!("{:?}", value);
    }

    #[test]
    fn serialize() {
        let data = vec![MetaValue::Valid, MetaValue::Empty, MetaValue::Deleted];
        let bytes = data.serialize();
        let other = Vec::<MetaValue>::deserialize(&bytes).unwrap();

        assert_eq!(data, other);
    }

    #[test]
    fn serialized_size() {
        assert_eq!(MetaValue::serialized_size(), 1);
    }
}
