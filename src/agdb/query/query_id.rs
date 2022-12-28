#[derive(Clone)]
pub enum QueryId {
    Id(u64),
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

impl From<u64> for QueryId {
    fn from(value: u64) -> Self {
        Self::Id(value)
    }
}
