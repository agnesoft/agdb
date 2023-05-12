use super::graph_index::GraphIndex;
use crate::db::db_error::DbError;

pub trait GraphData {
    fn capacity(&self) -> Result<u64, DbError>;
    fn commit(&mut self, id: u64) -> Result<(), DbError>;
    fn free_index(&self) -> Result<i64, DbError>;
    fn from(&self, index: &GraphIndex) -> Result<i64, DbError>;
    #[allow(clippy::wrong_self_convention)]
    fn from_meta(&self, index: &GraphIndex) -> Result<i64, DbError>;
    fn grow(&mut self) -> Result<(), DbError>;
    fn node_count(&self) -> Result<u64, DbError>;
    fn set_from(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError>;
    fn set_from_meta(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError>;
    fn set_node_count(&mut self, count: u64) -> Result<(), DbError>;
    fn set_to(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError>;
    fn set_to_meta(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError>;
    fn to(&self, index: &GraphIndex) -> Result<i64, DbError>;
    fn to_meta(&self, index: &GraphIndex) -> Result<i64, DbError>;
    fn transaction(&mut self) -> u64;
}
