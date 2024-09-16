/// Wrapper around `Vec<String>` to provide
/// several convenient conversions for the
/// [`QueryBuilder`].
pub struct QueryAliases(pub Vec<String>);

impl<T: Into<String>> From<Vec<T>> for QueryAliases {
    fn from(value: Vec<T>) -> Self {
        QueryAliases(value.into_iter().map(|v| v.into()).collect())
    }
}

impl<T: Into<String> + Clone> From<&[T]> for QueryAliases {
    fn from(value: &[T]) -> Self {
        QueryAliases(value.iter().map(|v| v.clone().into()).collect())
    }
}

impl<T: Into<String>, const N: usize> From<[T; N]> for QueryAliases {
    fn from(value: [T; N]) -> Self {
        QueryAliases(value.into_iter().map(|v| v.into()).collect())
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

impl From<&String> for QueryAliases {
    fn from(value: &String) -> Self {
        QueryAliases(vec![value.clone()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_aliases() {
        let _aliases = QueryAliases::from(vec!["a".to_string()]);
        let _aliases = QueryAliases::from(["a", "b"]);
        let _aliases = QueryAliases::from(["a", "b"].as_slice());
        let _aliases = QueryAliases::from("a");
        let _aliases = QueryAliases::from("a".to_string());
        let _aliases = QueryAliases::from(&"a".to_string());
    }
}
