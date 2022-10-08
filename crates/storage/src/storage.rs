use agdb_db_error::DbError;
use agdb_serialize::Serialize;

pub trait Storage {
    fn commit(&mut self) -> Result<(), DbError>;
    fn insert<V: Serialize>(&mut self, value: &V) -> Result<i64, DbError>;
    fn insert_at<V: Serialize>(
        &mut self,
        index: i64,
        offset: u64,
        value: &V,
    ) -> Result<(), DbError>;
    fn move_at(
        &mut self,
        index: i64,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError>;
    fn remove(&mut self, index: i64) -> Result<(), DbError>;
    fn resize_value(&mut self, index: i64, new_size: u64) -> Result<(), DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn size(&mut self) -> Result<u64, DbError>;
    fn transaction(&mut self);
    fn value<V: Serialize>(&mut self, index: i64) -> Result<V, DbError>;
    fn value_at<V: Serialize>(&mut self, index: i64, offset: u64) -> Result<V, DbError>;
    fn value_size(&self, index: i64) -> Result<u64, DbError>;
}
