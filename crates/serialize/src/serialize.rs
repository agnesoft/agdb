use agdb_db_error::DbError;

pub trait Serialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError>;
    fn serialize(&self) -> Vec<u8>;

    fn serialized_size() -> u64 {
        std::mem::size_of::<Self>() as u64
    }
}
