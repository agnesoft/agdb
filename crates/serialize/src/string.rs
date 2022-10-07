use crate::Serialize;
use db_error::DbError;

impl Serialize for String {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(String::from_utf8(bytes.to_vec())?)
    }

    fn serialize(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn serialized_size() -> u64 {
        0
    }
}
