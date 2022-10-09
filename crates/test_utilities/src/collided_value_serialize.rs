use crate::collided_value::CollidedValue;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;

impl<T> Serialize for CollidedValue<T>
where
    T: Serialize,
{
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            value: T::deserialize(bytes)?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        self.value.serialize()
    }
}
