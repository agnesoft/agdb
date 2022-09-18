#[derive(Debug)]
pub struct DbError {
    pub description: String,
    pub cause: Option<Box<DbError>>,
    pub source_location: std::panic::Location<'static>,
}

#[allow(dead_code)]
impl DbError {
    pub(crate) fn caused_by(mut self, error: DbError) -> DbError {
        self.cause = Some(Box::new(error));

        self
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = self.source_location.to_string().replace('\\', "/");
        write!(f, "{} (at {})", self.description, location)
    }
}

impl From<std::io::Error> for DbError {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
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

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Some(cause) = &self.cause {
            return Some(cause);
        }

        None
    }
}

impl PartialEq for DbError {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description && self.cause == other.cause
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn derived_from_debug() {
        let error = DbError::from("error");

        format!("{:?}", error);
    }

    #[test]
    fn derived_from_display() {
        let file = file!();
        let col__ = column!();
        let line = line!();
        let error = DbError::from("file not found");

        assert_eq!(
            error.to_string(),
            format!(
                "file not found (at {}:{}:{})",
                file.replace('\\', "/"),
                line + 1,
                col__
            )
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        let left = DbError::from(std::io::Error::from(std::io::ErrorKind::NotFound));
        let right = DbError::from(std::io::Error::from(std::io::ErrorKind::NotFound));

        assert_eq!(left, right);
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
        let _error = DbError::from(std::io::Error::from(std::io::ErrorKind::NotFound));
    }

    #[test]
    fn from_utf8_error() {
        let _error = DbError::from(String::from_utf8(vec![0xdf, 0xff]).unwrap_err());
    }

    #[test]
    fn source() {
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
    fn source_none() {
        let error = DbError::from("file not found");

        assert!(error.source().is_none());
    }
}
