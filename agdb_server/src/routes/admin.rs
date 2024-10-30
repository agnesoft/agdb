pub(crate) mod db;
pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::DbPool;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use agdb_api::AdminStatus;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use tokio::sync::broadcast::Sender;

#[utoipa::path(post,
    path = "/api/v1/admin/shutdown",
    operation_id = "admin_shutdown",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 202, description = "server is shutting down"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "admin only"),
    )
)]
pub(crate) async fn shutdown(
    _admin_id: AdminId,
    State(shutdown_sender): State<Sender<()>>,
) -> StatusCode {
    match shutdown_sender.send(()) {
        Ok(_) => StatusCode::ACCEPTED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[utoipa::path(get,
    path = "/api/v1/admin/status",
    operation_id = "admin_status",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 200, description = "Server is ready", body = AdminStatus),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn status(
    _admin_id: AdminId,
    State(config): State<Config>,
    State(db_pool): State<DbPool>,
) -> ServerResponse<(StatusCode, Json<AdminStatus>)> {
    let status = db_pool.status(&config).await?;
    Ok((StatusCode::OK, Json(status)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::DbId;

    #[tokio::test]
    async fn shutdown_test() -> anyhow::Result<()> {
        let (shutdown_sender, _shutdown_receiver) = tokio::sync::broadcast::channel::<()>(1);

        let status = shutdown(AdminId(DbId(0)), State(shutdown_sender)).await;

        assert_eq!(status, StatusCode::ACCEPTED);
        Ok(())
    }

    #[tokio::test]
    async fn bad_shutdown() -> anyhow::Result<()> {
        let shutdown_sender = Sender::<()>::new(1);

        let status = shutdown(AdminId(DbId(0)), State(shutdown_sender)).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }
}
