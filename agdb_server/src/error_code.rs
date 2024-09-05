use crate::server_error::ServerError;
use axum::http::StatusCode;

pub(crate) enum ErrorCode {
    PasswordTooShort,
    NameTooShort,
    UserExists,
    DbExists,
    DbInvalid,
    QueryError,
    ClusterHashMismatch,
    TermMismatch,
    LeaderExists,
    AlreadyVoted,
}

impl From<ErrorCode> for StatusCode {
    fn from(value: ErrorCode) -> Self {
        (&value).into()
    }
}

impl From<&ErrorCode> for StatusCode {
    fn from(value: &ErrorCode) -> Self {
        StatusCode::from_u16(match value {
            ErrorCode::PasswordTooShort => 461,
            ErrorCode::NameTooShort => 462,
            ErrorCode::UserExists => 463,
            ErrorCode::DbExists => 465,
            ErrorCode::DbInvalid => 467,
            ErrorCode::QueryError => 470,
            ErrorCode::ClusterHashMismatch => 480,
            ErrorCode::TermMismatch => 481,
            ErrorCode::LeaderExists => 482,
            ErrorCode::AlreadyVoted => 483,
        })
        .unwrap()
    }
}

impl From<ErrorCode> for ServerError {
    fn from(value: ErrorCode) -> Self {
        ServerError::new((&value).into(), value.as_str())
    }
}

impl ErrorCode {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            ErrorCode::PasswordTooShort => "password too short (<8)",
            ErrorCode::NameTooShort => "name too short (<3)",
            ErrorCode::UserExists => "user exists",
            ErrorCode::DbExists => "db already exists",
            ErrorCode::DbInvalid => "db invalid",
            ErrorCode::QueryError => "query error",
            ErrorCode::ClusterHashMismatch => "cluster hash mismatch",
            ErrorCode::TermMismatch => "term mismatch",
            ErrorCode::LeaderExists => "leader exists",
            ErrorCode::AlreadyVoted => "already voted",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str() {
        assert_eq!(
            ErrorCode::PasswordTooShort.as_str(),
            "password too short (<8)"
        );
        assert_eq!(ErrorCode::NameTooShort.as_str(), "name too short (<3)");
        assert_eq!(ErrorCode::UserExists.as_str(), "user exists");
        assert_eq!(ErrorCode::DbExists.as_str(), "db already exists");
        assert_eq!(ErrorCode::DbInvalid.as_str(), "db invalid");
        assert_eq!(ErrorCode::QueryError.as_str(), "query error");
        assert_eq!(
            ErrorCode::ClusterHashMismatch.as_str(),
            "cluster hash mismatch"
        );
        assert_eq!(ErrorCode::TermMismatch.as_str(), "term mismatch");
        assert_eq!(ErrorCode::LeaderExists.as_str(), "leader exists");
    }
}
