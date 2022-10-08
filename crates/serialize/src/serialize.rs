use agdb_db_error::DbError;
use std::mem::size_of;

pub trait Serialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError>;
    fn serialize(&self) -> Vec<u8>;

    fn serialized_size() -> u64 {
        size_of::<Self>() as u64
    }
}
