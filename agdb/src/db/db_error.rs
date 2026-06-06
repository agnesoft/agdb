use std::array::TryFromSliceError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FMTResult;
use std::io::Error as IOError;
use std::num::TryFromIntError;
use std::panic::Location;
use std::string::FromUtf8Error;
use std::sync::PoisonError;

use crate::type_def::TypeDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub enum DbErrorType {
    DbCreate,
    InvalidIndex,
    NotAllowed,
    NotEnoughData,
    NotFound,
    OutOfBounds,
    TypeError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub enum DbErrorCategory {
    Collections,
    Db,
    Graph,
    Query,
    Serialization,
    Storage,
}

/// Universal `agdb` database error. It represents
/// any error caused by the database processing such as
/// loading a database, running queries, writing data etc.
#[derive(Debug)]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct DbError {
    /// Error description
    pub description: String,

    /// Error category, e.g. graph, db, storage etc.
    pub category: DbErrorCategory,

    /// Error type, e.g. not found, not enough data etc.
    pub ty: DbErrorType,

    /// Optional error that caused this error
    pub cause: Option<Box<DbError>>,

    /// Location where the error originated in the sources
    pub source_location: Location<'static>,
}

#[cfg(feature = "api")]
impl TypeDefinition for Location<'_> {
    fn type_def() -> agdb::type_def::Type {
        agdb::type_def::Type::Struct(agdb::type_def::Struct {
            name: "Location",
            generics: &[],
            fields: &[
                agdb::type_def::Variable {
                    name: "file",
                    ty: Some(String::type_def),
                },
                agdb::type_def::Variable {
                    name: "line",
                    ty: Some(u32::type_def),
                },
                agdb::type_def::Variable {
                    name: "column",
                    ty: Some(u32::type_def),
                },
            ],
            impl_defs: || vec![],
        })
    }
}

impl DbError {
    /// Creates an error with an explicit kind.
    #[track_caller]
    pub fn new(category: DbErrorCategory, ty: DbErrorType, description: impl Into<String>) -> Self {
        DbError {
            description: description.into(),
            category,
            ty,
            cause: None,
            source_location: *Location::caller(),
        }
    }

    #[track_caller]
    pub fn collections(ty: DbErrorType, description: impl Into<String>) -> Self {
        Self::new(DbErrorCategory::Collections, ty, description)
    }

    #[track_caller]
    pub fn db(ty: DbErrorType, description: impl Into<String>) -> Self {
        Self::new(DbErrorCategory::Db, ty, description)
    }

    #[track_caller]
    pub fn graph(ty: DbErrorType, description: impl Into<String>) -> Self {
        Self::new(DbErrorCategory::Graph, ty, description)
    }

    #[track_caller]
    pub fn query(ty: DbErrorType, description: impl Into<String>) -> Self {
        Self::new(DbErrorCategory::Query, ty, description)
    }

    #[track_caller]
    pub fn serialization(ty: DbErrorType, description: impl Into<String>) -> Self {
        Self::new(DbErrorCategory::Serialization, ty, description)
    }

    #[track_caller]
    pub fn storage(ty: DbErrorType, description: impl Into<String>) -> Self {
        Self::new(DbErrorCategory::Storage, ty, description)
    }

    /// Sets the `cause` of this error to `error`.
    pub fn caused_by(mut self, error: Self) -> Self {
        self.cause = Some(Box::new(error));

        self
    }
}

impl Display for DbErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        match self {
            DbErrorType::DbCreate => write!(f, "DbCreate"),
            DbErrorType::InvalidIndex => write!(f, "InvalidIndex"),
            DbErrorType::NotEnoughData => write!(f, "NotEnoughData"),
            DbErrorType::NotFound => write!(f, "NotFound"),
            DbErrorType::OutOfBounds => write!(f, "OutOfBounds"),
            DbErrorType::NotAllowed => write!(f, "NotAllowed"),
            DbErrorType::TypeError => write!(f, "TypeError"),
        }
    }
}

impl Display for DbErrorCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        match self {
            DbErrorCategory::Collections => write!(f, "Collections"),
            DbErrorCategory::Db => write!(f, "Db"),
            DbErrorCategory::Graph => write!(f, "Graph"),
            DbErrorCategory::Query => write!(f, "Query"),
            DbErrorCategory::Serialization => write!(f, "Serialization"),
            DbErrorCategory::Storage => write!(f, "Storage"),
        }
    }
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        let location = self.source_location.to_string().replace('\\', "/");
        if let Some(cause) = &self.cause {
            write!(
                f,
                "[{}:{}] {} (at {})\ncaused by\n  {}",
                self.category, self.ty, self.description, location, cause
            )
        } else {
            write!(
                f,
                "[{}:{}] {} (at {})",
                self.category, self.ty, self.description, location
            )
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

impl<T> From<PoisonError<T>> for DbError {
    #[track_caller]
    fn from(value: PoisonError<T>) -> Self {
        DbError::db(DbErrorType::TypeError, value.to_string())
    }
}

impl From<IOError> for DbError {
    #[track_caller]
    fn from(error: IOError) -> Self {
        DbError::db(DbErrorType::TypeError, error.to_string())
    }
}

impl From<FromUtf8Error> for DbError {
    #[track_caller]
    fn from(error: FromUtf8Error) -> Self {
        DbError::db(DbErrorType::TypeError, error.to_string())
    }
}

impl From<TryFromSliceError> for DbError {
    #[track_caller]
    fn from(error: TryFromSliceError) -> Self {
        DbError::db(DbErrorType::NotEnoughData, error.to_string())
    }
}

impl From<TryFromIntError> for DbError {
    #[track_caller]
    fn from(error: TryFromIntError) -> Self {
        DbError::db(DbErrorType::TypeError, error.to_string())
    }
}

impl PartialEq for DbError {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description && self.ty == other.ty && self.cause == other.cause
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn derived_from_debug() {
        let error = DbError::db(DbErrorType::NotEnoughData, "error");
        let _ = format!("{error:?}");
    }

    #[test]
    fn derived_from_display() {
        let file = file!();
        let col__ = column!();
        let line = line!();
        let error = DbError::db(DbErrorType::NotEnoughData, "outer error");
        assert_eq!(
            error.to_string(),
            format!(
                "[Db:NotEnoughData] outer error (at {}:{}:{})",
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
        let mut error = DbError::db(DbErrorType::NotEnoughData, "outer error");
        let inner_column_adjusted = column!();
        let inner_line = line!();
        error.cause = Some(Box::new(DbError::db(
            DbErrorType::NotEnoughData,
            "inner error",
        )));

        assert_eq!(
            error.to_string(),
            format!(
                "[Db:NotEnoughData] outer error (at {}:{}:{})\ncaused by\n  [Db:NotEnoughData] inner error (at {}:{}:{})",
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
        let error = DbError::db(DbErrorType::NotEnoughData, "open error");
        let new_error = DbError::db(DbErrorType::NotEnoughData, "file not found").caused_by(error);
        assert_eq!(
            new_error.source().unwrap().to_string(),
            format!(
                "[Db:NotEnoughData] open error (at {}:{}:{})",
                file.replace('\\', "/"),
                line + 1,
                col__
            )
        );
    }

    #[test]
    fn caused_by() {
        let error = DbError::db(DbErrorType::NotEnoughData, "file not found");
        let new_error = DbError::db(DbErrorType::NotEnoughData, "open error").caused_by(error);
        assert_eq!(
            new_error.cause,
            Some(Box::new(DbError::db(
                DbErrorType::NotEnoughData,
                "file not found"
            )))
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
        let error = DbError::db(DbErrorType::NotEnoughData, "file not found");

        assert!(error.source().is_none());
    }

    #[test]
    fn from_poison_error() {
        let _ = DbError::from(PoisonError::<i32>::new(0));
    }

    #[test]
    fn from_io_error_has_kind() {
        let error = DbError::from(IOError::from(ErrorKind::NotFound));
        assert_eq!(error.ty, DbErrorType::TypeError);
    }

    #[test]
    fn from_string_has_no_kind() {
        let error = DbError::db(DbErrorType::NotEnoughData, "error");

        assert_eq!(error.ty, DbErrorType::NotEnoughData);
    }
}
