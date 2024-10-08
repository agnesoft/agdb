use std::array::TryFromSliceError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FMTResult;
use std::io::Error as IOError;
use std::num::TryFromIntError;
use std::panic::Location;
use std::string::FromUtf8Error;

/// Universal `agdb` database error. It represents
/// any error caused by the database processing such as
/// loading a database, writing data etc.
#[derive(Debug)]
pub struct DbError {
    /// Error description
    pub description: String,

    /// Optional error that caused this error
    pub cause: Option<Box<DbError>>,

    /// Location where the error originated in the sources
    pub source_location: Location<'static>,
}

impl DbError {
    /// Sets the `cause` of this error to `error`.
    pub fn caused_by(mut self, error: Self) -> Self {
        self.cause = Some(Box::new(error));

        self
    }
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        let location = self.source_location.to_string().replace('\\', "/");
        if let Some(cause) = &self.cause {
            write!(
                f,
                "{} (at {})\ncaused by\n  {}",
                self.description, location, cause
            )
        } else {
            write!(f, "{} (at {})", self.description, location)
        }
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
        let _ = format!("{error:?}");
    }

    #[test]
    fn derived_from_display() {
        let file = file!();
        let col__ = column!();
        let line = line!();
        let error = DbError::from("outer error");
        assert_eq!(
            error.to_string(),
            format!(
                "outer error (at {}:{}:{})",
                file.replace('\\', "/"),
                line + 1,
                col__
            )
        );
    }

    #[test]
    fn derived_from_display_cause() {
        let file = file!();
        let column___ = column!();
        let line = line!();
        let mut error = DbError::from("outer error");
        let inner_column_adjusted = column!();
        let inner_line = line!();
        error.cause = Some(Box::new(DbError::from("inner error")));

        assert_eq!(
            error.to_string(),
            format!(
                "outer error (at {}:{}:{})\ncaused by\n  inner error (at {}:{}:{})",
                file.replace('\\', "/"),
                line + 1,
                column___,
                file.replace('\\', "/"),
                inner_line + 1,
                inner_column_adjusted,
            )
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        let left = DbError::from(IOError::from(ErrorKind::NotFound));
        let right = DbError::from(IOError::from(ErrorKind::NotFound));
        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_error() {
        let file = file!();
        let col__ = column!();
        let line = line!();
        let error = DbError::from("file not found");
        let new_error = DbError::from("open error").caused_by(error);
        assert_eq!(
            new_error.source().unwrap().to_string(),
            format!(
                "file not found (at {}:{}:{})",
                file.replace('\\', "/"),
                line + 1,
                col__
            )
        );
    }

    #[test]
    fn caused_by() {
        let error = DbError::from("file not found");
        let new_error = DbError::from("open error").caused_by(error);
        assert_eq!(
            new_error.cause,
            Some(Box::new(DbError::from("file not found")))
        );
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
