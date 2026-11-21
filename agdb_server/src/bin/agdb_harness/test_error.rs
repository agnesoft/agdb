use agdb_api::AgdbApiError;
use std::env::VarError;

#[derive(Debug)]
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

impl From<agdb::DbError> for TestError {
    #[track_caller]
    fn from(error: agdb::DbError) -> Self {
        TestError {
            description: error.to_string(),
        }
    }
}
