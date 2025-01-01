use crate::DbId;

/// Database id used in queries that lets
/// you refer to a database element as numerical
/// id or a string alias.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
pub enum QueryId {
    /// Numerical id as [`DbId`]
    Id(DbId),

    /// String alias
    Alias(String),
}

impl From<&str> for QueryId {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<String> for QueryId {
    fn from(value: String) -> Self {
        Self::Alias(value)
    }
}

impl From<&String> for QueryId {
    fn from(value: &String) -> Self {
        Self::Alias(value.clone())
    }
}

impl From<i64> for QueryId {
    fn from(value: i64) -> Self {
        Self::Id(DbId(value))
    }
}

impl From<DbId> for QueryId {
    fn from(value: DbId) -> Self {
        Self::Id(value)
    }
}

impl Default for QueryId {
    fn default() -> Self {
        Self::Id(DbId::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_db_id() {
        let _ = QueryId::from(DbId(0));
    }
}
