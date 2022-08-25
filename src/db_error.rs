#[allow(dead_code)]
pub(crate) enum DbError {
    Storage(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage() {
        let _error = DbError::Storage("error".to_string());
    }
}
