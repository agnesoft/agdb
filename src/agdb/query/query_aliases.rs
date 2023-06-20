pub struct QueryAliases(pub Vec<String>);

impl From<Vec<String>> for QueryAliases {
    fn from(value: Vec<String>) -> Self {
        QueryAliases(value)
    }
}

impl From<Vec<&str>> for QueryAliases {
    fn from(value: Vec<&str>) -> Self {
        QueryAliases(value.iter().map(|v| v.to_string()).collect())
    }
}

impl From<&str> for QueryAliases {
    fn from(value: &str) -> Self {
        QueryAliases(vec![value.to_string()])
    }
}

impl From<String> for QueryAliases {
    fn from(value: String) -> Self {
        QueryAliases(vec![value])
    }
}
