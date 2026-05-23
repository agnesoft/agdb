pub(crate) mod db;
pub(crate) mod user;

use crate::config::Config;
use crate::logger;
use crate::server_db::ServerDb;
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
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::sync::broadcast::Sender;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Deserialize, IntoParams, ToSchema, agdb::TypeDef)]
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
) -> ServerResponse<(StatusCode, Json<AdminStatus>)> {
    Ok((
        StatusCode::OK,
        Json(AdminStatus {
            uptime: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - config.start_time,
            dbs: server_db.db_count().await?,
            users: server_db.user_count().await?,
            logged_in_users: server_db.users_with_token().await?,
            size: get_size(&config.data_dir).await?,
            memory: get_process_memory(),
            log_level: logger::current_level(),
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
    request: Query<SetLogLevelRequest>,
) -> StatusCode {
    logger::set_level(request.new_level);
    crate::info!("Log level changed to: {}", request.new_level);
    StatusCode::OK
}

#[cfg(target_os = "linux")]
fn get_process_memory() -> u64 {
    if let Ok(status_content) = std::fs::read_to_string("/proc/self/status")
        && let Some((_, vmrss)) = status_content.split_once("VmRSS:")
        && let Some((vmrss_value, _)) = vmrss.trim().split_once(' ')
    {
        vmrss_value.trim().parse::<u64>().unwrap_or_default() * 1024
    }

    0
}

#[cfg(target_os = "windows")]
fn get_process_memory() -> u64 {
    use windows_sys::Win32::System::ProcessStatus::K32GetProcessMemoryInfo;
    use windows_sys::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    let mut counters = PROCESS_MEMORY_COUNTERS {
        cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        ..Default::default()
    };
    let counter_ptr = &mut counters as *mut _ as *mut _;

    // SAFETY: We call Win32 APIs with a valid pseudo-handle for the current process,
    // a properly initialized struct, and the correct struct size.
    unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), counter_ptr, counters.cb) };

    counters.WorkingSetSize as u64
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn get_process_memory() -> u64 {
    0
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
