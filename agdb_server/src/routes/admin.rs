pub(crate) mod db;
pub(crate) mod user;

use crate::config::Config;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use crate::utilities::get_size;
use agdb_api::AdminStatus;
use agdb_api::LogLevelFilter;
use axum::Json;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::sync::broadcast::Sender;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;
use tracing_subscriber::reload::Handle;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Deserialize, IntoParams, ToSchema, agdb::TypeDefImpl)]
#[into_params(parameter_in = Query)]
pub struct SetLogLevelRequest {
    pub new_level: LogLevelFilter,
}

#[utoipa::path(post,
    path = "/api/v1/admin/shutdown",
    operation_id = "admin_shutdown",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 202, description = "server is shutting down"),
         (status = 401, description = "unauthorized"),
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
         (status = 200, description = "server is ready", body = AdminStatus),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn status(
    _admin_id: AdminId,
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    State(handle): State<Arc<Handle<EnvFilter, Registry>>>,
) -> ServerResponse<(StatusCode, Json<AdminStatus>)> {
    Ok((
        StatusCode::OK,
        Json(AdminStatus {
            uptime: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - config.start_time,
            dbs: server_db.db_count().await?,
            users: server_db.user_count().await?,
            logged_in_users: server_db.user_token_count().await?,
            size: get_size(&config.data_dir).await?,
            log_level: handle
                .with_current(|f| f.to_string().as_str().try_into().unwrap_or_default())
                .unwrap_or_default(),
        }),
    ))
}

#[utoipa::path(post,
    path = "/api/v1/admin/set_log_level",
    operation_id = "set_log_level",
    tag = "agdb",
    security(("Token" = [])),
    params(
        SetLogLevelRequest
    ),
    responses(
         (status = 200, description = "log level changed"),
         (status = 400, description = "invalid log level"),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn set_log_level(
    _admin_id: AdminId,
    State(handle): State<Arc<Handle<EnvFilter, Registry>>>,
    request: Query<SetLogLevelRequest>,
) -> Result<(), ServerError> {
    handle
        .modify(|filter| {
            *filter = EnvFilter::new(request.new_level.to_string());
        })
        .map_err(|e| ServerError::new(StatusCode::INTERNAL_SERVER_ERROR, &format!("{e:?}")))?;
    tracing::info!("Log level changed to: {}", request.new_level);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shutdown_test() -> anyhow::Result<()> {
        let (shutdown_sender, _shutdown_receiver) = tokio::sync::broadcast::channel::<()>(1);

        let status = shutdown(AdminId(), State(shutdown_sender)).await;

        assert_eq!(status, StatusCode::ACCEPTED);
        Ok(())
    }

    #[tokio::test]
    async fn bad_shutdown() -> anyhow::Result<()> {
        let shutdown_sender = Sender::<()>::new(1);

        let status = shutdown(AdminId(), State(shutdown_sender)).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }
}
