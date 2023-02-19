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

impl From<String> for QueryError {
    fn from(value: String) -> Self {
        Self { description: value }
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
    fn derived_from_debug() {
        format!(
            "{:?}",
            QueryError {
                description: String::new()
            }
        );
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
