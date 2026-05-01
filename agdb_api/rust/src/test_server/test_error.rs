use crate::AgdbApiError;
use std::env::VarError;

#[derive(Debug, agdb::TypeDefImpl)]
pub struct TestError {
    description: String,
}

impl TestError {
    pub(crate) fn new(description: impl Into<String>) -> Self {
        TestError {
            description: description.into(),
        }
    }
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for TestError {}

/// Early-returns an `Err(TestError)` from the current function.
/// Supports the same format-string syntax as `format!`.
macro_rules! bail {
    ($fmt:literal $(,)?) => {
        return Err(crate::test_server::test_error::TestError::new(format!($fmt)))
    };
    ($fmt:literal, $($arg:tt)*) => {
        return Err(crate::test_server::test_error::TestError::new(format!($fmt, $($arg)*)))
    };
}
pub(crate) use bail;

impl From<std::io::Error> for TestError {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        TestError {
            description: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for TestError {
    #[track_caller]
    fn from(error: reqwest::Error) -> Self {
        TestError {
            description: error.to_string(),
        }
    }
}

impl From<AgdbApiError> for TestError {
    #[track_caller]
    fn from(error: AgdbApiError) -> Self {
        TestError {
            description: error.to_string(),
        }
    }
}

impl From<VarError> for TestError {
    #[track_caller]
    fn from(error: VarError) -> Self {
        TestError {
            description: error.to_string(),
        }
    }
}

impl From<tokio::task::JoinError> for TestError {
    #[track_caller]
    fn from(error: tokio::task::JoinError) -> Self {
        TestError {
            description: error.to_string(),
        }
    }
}
