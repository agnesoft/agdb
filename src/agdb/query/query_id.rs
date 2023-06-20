use crate::DbId;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryId {
    Id(DbId),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_db_id() {
        let _ = QueryId::from(DbId(0));
    }
}
