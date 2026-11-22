pub(crate) mod user;

use crate::action::dashboard_public_set::DashboardPublicSet;
use crate::action::dashboard_remove::DashboardRemove;
use crate::action::dashboard_rename::DashboardRename;
use crate::action::dashboard_save::DashboardSave;
use crate::cluster::Cluster;
use crate::server_db::ServerDashboard;
use crate::server_db::ServerDb;
use crate::server_db::dashboard_exists;
use crate::server_error::CommitIndexHeader;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::server_error::ServerResult;
use crate::server_error::permission_denied;
use crate::user_id::UserId;
use crate::user_id::UserName;
use agdb_api::Dashboard;
use agdb_api::DashboardConfig;
use agdb_api::DashboardSummary;
use agdb_api::DbUserRole;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use utoipa::IntoParams;
use utoipa::ToSchema;

/// Largest accepted dashboard config (bytes).
pub(crate) const MAX_DASHBOARD_CONFIG: usize = 256 * 1024;
const MAX_DASHBOARD_NAME: usize = 64;

#[derive(Deserialize, IntoParams, ToSchema, agdb::TypeDef)]
#[into_params(parameter_in = Query)]
pub struct DashboardRenameParam {
    pub new_name: String,
}

#[derive(Deserialize, IntoParams, ToSchema, agdb::TypeDef)]
#[into_params(parameter_in = Query)]
pub struct DashboardPublicParam {
    /// Every user can view a public dashboard.
    pub public: bool,
}

fn validate_name(name: &str) -> ServerResult {
    let valid = !name.is_empty()
        && name.len() <= MAX_DASHBOARD_NAME
        // "." and ".." would be collapsed out of the URL path.
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');

    if valid {
        Ok(())
    } else {
        Err(ServerError::new(
            StatusCode::BAD_REQUEST,
            &format!(
                "invalid dashboard name '{name}': use 1-{MAX_DASHBOARD_NAME} characters [A-Za-z0-9_.-], not starting with '.'"
            ),
        ))
    }
}

pub(crate) fn validate_config(config: &str) -> ServerResult {
    if config.len() > MAX_DASHBOARD_CONFIG {
        return Err(ServerError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!("dashboard config too large (>{MAX_DASHBOARD_CONFIG} bytes)"),
        ));
    }
    Ok(())
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) fn dashboard_response(
    dashboard: ServerDashboard,
    role: DbUserRole,
    public: bool,
) -> Dashboard {
    Dashboard {
        owner: dashboard.owner,
        name: dashboard.name,
        config: dashboard.config,
        updated: dashboard.updated,
        role,
        public,
    }
}

/// Adds a dashboard for `owner` (an existing user); returns the commit index.
pub(crate) async fn add_dashboard(
    cluster: &Cluster,
    server_db: &ServerDb,
    owner: String,
    name: String,
    config: String,
) -> ServerResult<u64> {
    validate_name(&name)?;
    validate_config(&config)?;

    if server_db.dashboard(&owner, &name).await.is_ok() {
        return Err(dashboard_exists(&name));
    }

    let (commit_index, _result) = cluster
        .exec(DashboardSave {
            owner,
            name,
            config,
            create: true,
            updated: now(),
        })
        .await?;
    Ok(commit_index)
}

/// Replaces the config of an existing dashboard; returns the commit index.
pub(crate) async fn save_dashboard(
    cluster: &Cluster,
    owner: String,
    name: String,
    config: String,
) -> ServerResult<u64> {
    let (commit_index, _result) = cluster
        .exec(DashboardSave {
            owner,
            name,
            config,
            create: false,
            updated: now(),
        })
        .await?;
    Ok(commit_index)
}

pub(crate) async fn rename_dashboard(
    cluster: &Cluster,
    server_db: &ServerDb,
    owner: String,
    name: String,
    new_name: String,
) -> ServerResult<u64> {
    validate_name(&new_name)?;

    if server_db.dashboard(&owner, &new_name).await.is_ok() {
        return Err(dashboard_exists(&new_name));
    }

    let (commit_index, _result) = cluster
        .exec(DashboardRename {
            owner,
            name,
            new_name,
            updated: now(),
        })
        .await?;
    Ok(commit_index)
}

pub(crate) async fn remove_dashboard(
    cluster: &Cluster,
    owner: String,
    name: String,
) -> ServerResult<u64> {
    let (commit_index, _result) = cluster.exec(DashboardRemove { owner, name }).await?;
    Ok(commit_index)
}

pub(crate) async fn set_dashboard_public(
    cluster: &Cluster,
    owner: String,
    name: String,
    public: bool,
) -> ServerResult<u64> {
    let (commit_index, _result) = cluster
        .exec(DashboardPublicSet {
            owner,
            name,
            public,
        })
        .await?;
    Ok(commit_index)
}

#[utoipa::path(get,
    path = "/api/v1/dashboard/list",
    operation_id = "dashboard_list",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = Vec<DashboardSummary>),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn list(
    user: UserId,
    State(server_db): State<ServerDb>,
) -> ServerResponse<(StatusCode, Json<Vec<DashboardSummary>>)> {
    let dashboards = server_db.user_dashboards(user.0).await?;
    Ok((StatusCode::OK, Json(dashboards)))
}

#[utoipa::path(get,
    path = "/api/v1/dashboard/{owner}/{name}",
    operation_id = "dashboard_get",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
    ),
    responses(
         (status = 200, description = "ok", body = Dashboard),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "dashboard not found"),
    )
)]
pub(crate) async fn get(
    user: UserId,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<Dashboard>)> {
    let access = server_db.user_dashboard(user.0, &owner, &name).await?;
    Ok((
        StatusCode::OK,
        Json(dashboard_response(
            access.dashboard,
            access.role,
            access.public,
        )),
    ))
}

#[utoipa::path(post,
    path = "/api/v1/dashboard/{owner}/{name}/add",
    operation_id = "dashboard_add",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
    ),
    request_body = DashboardConfig,
    responses(
         (status = 201, description = "dashboard added"),
         (status = 400, description = "invalid dashboard name"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "cannot add dashboard to another user"),
         (status = 409, description = "dashboard already exists"),
         (status = 413, description = "dashboard config too large"),
    )
)]
pub(crate) async fn add(
    username: UserName,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
    Json(request): Json<DashboardConfig>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    if username.0 != owner {
        return Err(permission_denied("cannot add dashboard to another user"));
    }

    let commit_index = add_dashboard(&cluster, &server_db, owner, name, request.config).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(put,
    path = "/api/v1/dashboard/{owner}/{name}/save",
    operation_id = "dashboard_save",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
    ),
    request_body = DashboardConfig,
    responses(
         (status = 201, description = "dashboard saved"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "write rights required"),
         (status = 404, description = "dashboard not found"),
         (status = 413, description = "dashboard config too large"),
    )
)]
pub(crate) async fn save(
    user: UserId,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
    Json(request): Json<DashboardConfig>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    // Users without access learn nothing about the dashboard (404) first.
    let access = server_db.user_dashboard(user.0, &owner, &name).await?;
    validate_config(&request.config)?;

    if access.role == DbUserRole::Read {
        return Err(permission_denied("write rights required"));
    }

    let commit_index = save_dashboard(&cluster, owner, name, request.config).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/dashboard/{owner}/{name}/rename",
    operation_id = "dashboard_rename",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
        DashboardRenameParam,
    ),
    responses(
         (status = 201, description = "dashboard renamed"),
         (status = 400, description = "invalid new dashboard name"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "owner only"),
         (status = 404, description = "dashboard not found"),
         (status = 409, description = "target dashboard already exists"),
    )
)]
pub(crate) async fn rename(
    user: UserId,
    username: UserName,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
    request: Query<DashboardRenameParam>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    let _ = server_db.user_dashboard(user.0, &owner, &name).await?;

    if username.0 != owner {
        return Err(permission_denied("owner only"));
    }

    let commit_index =
        rename_dashboard(&cluster, &server_db, owner, name, request.new_name.clone()).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(delete,
    path = "/api/v1/dashboard/{owner}/{name}/remove",
    operation_id = "dashboard_remove",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
    ),
    responses(
         (status = 204, description = "dashboard removed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "owner only"),
         (status = 404, description = "dashboard not found"),
    )
)]
pub(crate) async fn remove(
    user: UserId,
    username: UserName,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    let _ = server_db.user_dashboard(user.0, &owner, &name).await?;

    if username.0 != owner {
        return Err(permission_denied("owner only"));
    }

    let commit_index = remove_dashboard(&cluster, owner, name).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(put,
    path = "/api/v1/dashboard/{owner}/{name}/public",
    operation_id = "dashboard_public",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
        DashboardPublicParam,
    ),
    responses(
         (status = 201, description = "dashboard made public / private"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "owner only"),
         (status = 404, description = "dashboard not found"),
    )
)]
pub(crate) async fn public(
    user: UserId,
    username: UserName,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
    request: Query<DashboardPublicParam>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    let _ = server_db.user_dashboard(user.0, &owner, &name).await?;

    if username.0 != owner {
        return Err(permission_denied("owner only"));
    }

    let commit_index = set_dashboard_public(&cluster, owner, name, request.public).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_name_accepts_simple_names() {
        assert!(validate_name("ops").is_ok());
        assert!(validate_name("list").is_ok());
        assert!(validate_name("sales-2026_q1.v2").is_ok());
    }

    #[test]
    fn validate_name_rejects_invalid_names() {
        assert!(validate_name("").is_err());
        assert!(validate_name("has space").is_err());
        assert!(validate_name("a/b").is_err());
        assert!(validate_name(".").is_err());
        assert!(validate_name("..").is_err());
        assert!(validate_name(".hidden").is_err());
        assert!(validate_name(&"x".repeat(MAX_DASHBOARD_NAME + 1)).is_err());
    }

    #[test]
    fn validate_config_rejects_too_large() {
        assert!(validate_config("{}").is_ok());
        assert!(validate_config(&"x".repeat(MAX_DASHBOARD_CONFIG + 1)).is_err());
    }
}
