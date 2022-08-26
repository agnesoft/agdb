#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub(crate) enum DbError {
    Storage(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let _error = DbError::Storage("error".to_string());
        format!("{:?}", _error);
    }

    #[test]
    fn storage() {
        let _error = DbError::Storage("error".to_string());
    }
}
