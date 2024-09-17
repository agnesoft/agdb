use crate::db::db_element::DbElement;
use crate::DbError;
use crate::DbId;
use crate::DbUserValue;

/// Universal database result. Successful
/// execution of a query will always yield
/// this type. The `result` field is a numerical
/// representation of the result while the
/// `elements` are the list of `DbElement`s
/// with database ids and properties (key-value pairs).
#[derive(Debug, Default, Eq, PartialOrd, Ord, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QueryResult {
    /// Query result
    pub result: i64,

    /// List of elements yielded by the query
    /// possibly with a list of properties.
    pub elements: Vec<DbElement>,
}

impl QueryResult {
    pub fn ids(&self) -> Vec<DbId> {
        self.elements.iter().map(|e| e.id).collect()
    }
}

impl<T: DbUserValue<ValueType = T>> TryInto<Vec<T>> for QueryResult {
    type Error = DbError;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        let mut result = Vec::with_capacity(self.elements.len());
        self.elements
            .iter()
            .try_for_each(|e| -> Result<(), DbError> {
                result.push(T::from_db_element(e)?);
                Ok(())
            })?;
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", QueryResult::default());
    }

    #[test]
    fn derived_from_clone() {
        let result = QueryResult::default();
        let other = result.clone();
        assert_eq!(result, other);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(QueryResult::default(), QueryResult::default());
    }

    #[test]
    fn derived_from_partial_ord() {
        let result = QueryResult {
            result: 0,
            elements: vec![],
        };
        let other = QueryResult {
            result: 1,
            elements: vec![],
        };

        assert!(result < other);
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(
            QueryResult::default().cmp(&QueryResult::default()),
            std::cmp::Ordering::Equal
        );
    }
}
