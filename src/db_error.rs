#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub(crate) enum DbError {
    Storage(String),
}

impl From<std::io::Error> for DbError {
    fn from(error: std::io::Error) -> Self {
        DbError::Storage(error.to_string())
    }
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
    fn from_io_error() {
        let _error = DbError::from(std::io::Error::from(std::io::ErrorKind::NotFound));
    }
}
