use crate::db::db_error::DbError;
use crate::utilities::serialize::OldSerialize;
use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CollidedValue<T> {
    pub value: T,
}

impl<T> CollidedValue<T> {
    pub fn new(value: T) -> Self {
        CollidedValue { value }
    }
}

impl<T> OldSerialize for CollidedValue<T>
where
    T: OldSerialize,
{
    fn old_deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            value: T::old_deserialize(bytes)?,
        })
    }

    fn old_serialize(&self) -> Vec<u8> {
        self.value.old_serialize()
    }
}

impl<T> StableHash for CollidedValue<T> {
    fn stable_hash(&self) -> u64 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_clone() {
        let value = CollidedValue::new(1_i64);
        let other = value.clone();

        assert_eq!(value, other);
    }

    #[test]
    fn derived_from_debug() {
        let value = CollidedValue::new(1_i64);

        format!("{:?}", value);
    }

    #[test]
    fn serialize() {
        let value = CollidedValue::new(1_i64);
        let bytes = value.old_serialize();
        let other = CollidedValue::old_deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn stable_hash() {
        let value = CollidedValue::new(1_i64);

        assert_eq!(value.stable_hash(), 1_u64);
    }
}
