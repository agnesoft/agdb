#[derive(Debug, Clone, PartialEq)]
pub enum QueryId {
    Id(i64),
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
        Self::Id(value)
    }
}

impl QueryId {
    pub fn is_node(&self) -> bool {
        match self {
            QueryId::Id(id) => 0 < *id,
            QueryId::Alias(_) => true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_node() {
        assert!(QueryId::Id(1).is_node());
        assert!(!QueryId::Id(0).is_node());
        assert!(!QueryId::Id(-1).is_node());
    }
}
