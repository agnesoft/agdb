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

impl QueryId {
    pub fn is_node(&self) -> bool {
        match self {
            QueryId::Id(id) => 0 < id.0,
            QueryId::Alias(_) => true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_node() {
        assert!(QueryId::from(1).is_node());
        assert!(!QueryId::from(0).is_node());
        assert!(!QueryId::from(-1).is_node());
    }
}
