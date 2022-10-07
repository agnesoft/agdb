use crate::Serialize;
use db_error::DbError;

impl Serialize for Vec<u8> {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(bytes.to_vec())
    }

    fn serialize(&self) -> Vec<u8> {
        self.to_vec()
    }
}
