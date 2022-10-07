use crate::DbError;

impl From<std::io::Error> for DbError {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        DbError::from(error.to_string())
    }
}

impl From<std::string::FromUtf8Error> for DbError {
    #[track_caller]
    fn from(error: std::string::FromUtf8Error) -> Self {
        DbError::from(error.to_string())
    }
}

impl From<&str> for DbError {
    #[track_caller]
    fn from(description: &str) -> Self {
        DbError::from(description.to_string())
    }
}

impl From<String> for DbError {
    #[track_caller]
    fn from(description: String) -> Self {
        DbError {
            description,
            cause: None,
            source_location: *std::panic::Location::caller(),
        }
    }
}
