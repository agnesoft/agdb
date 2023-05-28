use std::array::TryFromSliceError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FMTResult;
use std::io::Error as IOError;
use std::num::TryFromIntError;
use std::panic::Location;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub struct DbError {
    pub description: String,
    pub cause: Option<Box<DbError>>,
    pub source_location: Location<'static>,
}

impl DbError {
    pub fn caused_by(mut self, error: Self) -> Self {
        self.cause = Some(Box::new(error));

        self
    }
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        let location = self.source_location.to_string().replace('\\', "/");
        write!(f, "{} (at {})", self.description, location)
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(cause) = &self.cause {
            return Some(cause);
        }

        None
    }
}

impl From<IOError> for DbError {
    #[track_caller]
    fn from(error: IOError) -> Self {
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

impl From<TryFromSliceError> for DbError {
    #[track_caller]
    fn from(error: TryFromSliceError) -> Self {
        DbError::from(error.to_string())
    }
}

impl From<TryFromIntError> for DbError {
    #[track_caller]
    fn from(error: TryFromIntError) -> Self {
        DbError::from(error.to_string())
    }
}

impl PartialEq for DbError {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description && self.cause == other.cause
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn derived_from_debug() {
        let error = DbError::from("error");
        format!("{error:?}");
    }

    #[test]
    fn caused_by() {
        let error = DbError::from("file not found");
        let new_error = DbError::from("open error").caused_by(error);
        let _ = new_error.source();
    }

    #[test]
    fn from_io_error() {
        let _error = DbError::from(IOError::from(ErrorKind::NotFound));
    }

    #[test]
    fn from_utf8_error() {
        let _error = DbError::from(String::from_utf8(vec![0xdf, 0xff]).unwrap_err());
    }

    #[test]
    fn from_try_from_slice_error() {
        let data = Vec::<u8>::new();
        let bytes: &[u8] = &data;
        let source_error = TryInto::<[u8; 8]>::try_into(bytes).unwrap_err();
        let _error = DbError::from(source_error);
    }

    #[test]
    fn from_try_int_error() {
        let source_error = TryInto::<u32>::try_into(u64::MAX).unwrap_err();
        let _error = DbError::from(source_error);
    }

    #[test]
    fn source_none() {
        let error = DbError::from("file not found");

        assert!(error.source().is_none());
    }
}
