pub(crate) mod admin;
pub(crate) mod cluster;
pub(crate) mod db;
#[cfg(feature = "studio")]
pub(crate) mod studio;
pub(crate) mod user;

use crate::server_error::ServerResult;
use axum::http::StatusCode;

#[utoipa::path(get,
    path = "/api/v1/status",
    operation_id = "status",
    tag = "agdb",
    responses(
         (status = 200, description = "Server is ready"),
    )
)]
pub(crate) async fn status() -> ServerResult<StatusCode> {
    Ok(StatusCode::OK)
}
