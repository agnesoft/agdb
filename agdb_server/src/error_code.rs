use crate::server_error::ServerError;
use axum::http::StatusCode;

pub(crate) enum ErrorCode {
    PasswordTooShort,
    NameTooShort,
    UserExists,
    UserNotFound,
    DbExists,
    DbNotFound,
    DbInvalid,
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
            ErrorCode::UserNotFound => 464,
            ErrorCode::DbExists => 465,
            ErrorCode::DbNotFound => 466,
            ErrorCode::DbInvalid => 467,
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
            ErrorCode::UserNotFound => "user not found",
            ErrorCode::DbExists => "db already exists",
            ErrorCode::DbNotFound => "db not found",
            ErrorCode::DbInvalid => "db invalid",
        }
    }
}
