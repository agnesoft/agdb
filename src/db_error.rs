#[allow(dead_code)]
pub(crate) enum DbError {
    Storage(String),
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use super::*;

    #[test]
    fn derived_from_debug() {
        let _error = DbError::Storage("error".to_string());
        format!("{:?}", _error);
    }

    #[test]
    fn from_io_error() {
        let _error = DbError::from(std::io::Error::from(ErrorKind::NotFound));
    }
}
