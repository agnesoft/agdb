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
}
