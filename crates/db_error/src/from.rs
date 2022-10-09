use crate::DbError;
use std::io::Error;
use std::panic::Location;
use std::string::FromUtf8Error;

impl From<Error> for DbError {
    #[track_caller]
    fn from(error: Error) -> Self {
        DbError::from(error.to_string())
    }
}

impl From<FromUtf8Error> for DbError {
    #[track_caller]
    fn from(error: FromUtf8Error) -> Self {
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
            source_location: *Location::caller(),
        }
    }
}
