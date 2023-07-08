use crate::db::db_error::DbError;
use std::sync::PoisonError;

/// Universal `query` error returned from all query operations.
/// It represents mainly errors from executing queries but the
/// cause of the error may be in exceptional cases a `DbError`.
/// Typically however it will contain description of a problem with
/// running a query such as "id/alias does not exist".
#[derive(Default, Debug, PartialEq)]
pub struct QueryError {
    pub description: String,
    pub cause: Option<DbError>,
}

impl From<DbError> for QueryError {
    fn from(value: DbError) -> Self {
        Self {
            description: format!("{value}"),
            cause: Some(value),
        }
    }
}

impl<T> From<PoisonError<T>> for QueryError {
    fn from(value: PoisonError<T>) -> Self {
        Self {
            description: value.to_string(),
            cause: None,
        }
    }
}

impl From<String> for QueryError {
    fn from(value: String) -> Self {
        Self {
            description: value,
            cause: None,
        }
    }
}

impl From<&str> for QueryError {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug_and_default() {
        format!("{:?}", QueryError::default());
    }

    #[test]
    fn derived_from_default() {
        let _ = QueryError::default();
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(QueryError::default(), QueryError::default());
    }

    #[test]
    fn from_db_error() {
        let _ = QueryError::from(DbError::from(""));
    }

    #[test]
    fn from_poison_error() {
        let _ = QueryError::from(PoisonError::<i32>::new(0));
    }

    #[test]
    fn from_string() {
        let _ = QueryError::from("".to_string());
    }

    #[test]
    fn from_str() {
        let _ = QueryError::from("");
    }
}
