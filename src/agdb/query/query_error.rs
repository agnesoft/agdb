use crate::DbError;
use std::sync::PoisonError;

#[derive(Default, Debug)]
pub struct QueryError {
    pub description: String,
}

impl From<DbError> for QueryError {
    fn from(value: DbError) -> Self {
        Self {
            description: format!("{value}"),
        }
    }
}

impl<T> From<PoisonError<T>> for QueryError {
    fn from(value: PoisonError<T>) -> Self {
        Self {
            description: value.to_string(),
        }
    }
}
