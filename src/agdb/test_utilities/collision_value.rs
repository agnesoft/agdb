use crate::db::db_error::DbError;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize_static::SerializeStatic;
use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CollisionValue<T> {
    pub value: T,
}

impl<T> CollisionValue<T> {
    pub fn new(value: T) -> Self {
        CollisionValue { value }
    }
}

impl<T> StableHash for CollisionValue<T> {
    fn stable_hash(&self) -> u64 {
        1
    }
}

impl<T> Serialize for CollisionValue<T>
where
    T: Serialize,
{
    fn serialize(&self) -> Vec<u8> {
        self.value.serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            value: T::deserialize(bytes)?,
        })
    }

    fn serialized_size(&self) -> u64 {
        self.value.serialized_size()
    }
}

impl<T> SerializeStatic for CollisionValue<T> where T: SerializeStatic {}

impl<T> StorageValue for CollisionValue<T>
where
    T: StorageValue,
{
    fn store<S: Storage>(&self, storage: &mut S) -> Result<Vec<u8>, DbError> {
        self.value.store(storage)
    }

    fn load<S: Storage>(storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            value: T::load(storage, bytes)?,
        })
    }

    fn remove<S: Storage>(storage: &mut S, bytes: &[u8]) -> Result<(), DbError> {
        T::remove(storage, bytes)
    }

    fn storage_len() -> u64 {
        T::storage_len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_clone() {
        let value = CollisionValue::new(1_i64);
        let other = value.clone();

        assert_eq!(value, other);
    }

    #[test]
    fn derived_from_debug() {
        let value = CollisionValue::new(1_i64);

        format!("{:?}", value);
    }

    #[test]
    fn serialize() {
        let value = CollisionValue::new(1_i64);
        let bytes = value.serialize();

        assert_eq!(bytes.len() as u64, value.serialized_size());

        let other = CollisionValue::deserialize(&bytes).unwrap();

        assert_eq!(value, other);
    }

    #[test]
    fn stable_hash() {
        let value = CollisionValue::new(1_i64);

        assert_eq!(value.stable_hash(), 1_u64);
    }
}
