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
