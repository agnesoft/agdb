use super::Storage;
use crate::utilities::serialize::Serialize;
use crate::DbError;

pub trait StorageValue: Serialize {
    fn store<S: Storage>(&self, storage: &mut S) -> Result<Vec<u8>, DbError>;
    fn load<S: Storage>(storage: &S, bytes: &[u8]) -> Result<Self, DbError>;
}
