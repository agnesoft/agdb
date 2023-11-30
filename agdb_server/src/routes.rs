use axum::http::StatusCode;

pub(crate) mod admin;
pub(crate) mod db;
pub(crate) mod user;

pub(crate) async fn test_error() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
