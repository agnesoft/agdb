use axum::http::StatusCode;

pub(crate) mod admin;
pub(crate) mod db;
pub(crate) mod user;

#[utoipa::path(get,
    path = "/api/v1/status",
    responses(
         (status = 200, description = "Server is ready"),
    )
)]
pub(crate) async fn status() -> StatusCode {
    StatusCode::OK
}

pub(crate) async fn test_error() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
