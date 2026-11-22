pub(crate) mod user;

use crate::cluster::Cluster;
use crate::routes::dashboard::DashboardPublicParam;
use crate::routes::dashboard::DashboardRenameParam;
use crate::routes::dashboard::add_dashboard;
use crate::routes::dashboard::dashboard_response;
use crate::routes::dashboard::remove_dashboard;
use crate::routes::dashboard::rename_dashboard;
use crate::routes::dashboard::save_dashboard;
use crate::routes::dashboard::set_dashboard_public;
use crate::routes::dashboard::validate_config;
use crate::server_db::ServerDb;
use crate::server_db::dashboard_not_found;
use crate::server_error::CommitIndexHeader;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use agdb_api::Dashboard;
use agdb_api::DashboardConfig;
use agdb_api::DashboardSummary;
use agdb_api::DbUserRole;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;

#[utoipa::path(get,
    path = "/api/v1/admin/dashboard/list",
    operation_id = "admin_dashboard_list",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = Vec<DashboardSummary>),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn list(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
) -> ServerResponse<(StatusCode, Json<Vec<DashboardSummary>>)> {
    Ok((StatusCode::OK, Json(server_db.dashboards().await?)))
}

#[utoipa::path(get,
    path = "/api/v1/admin/dashboard/{owner}/{name}",
    operation_id = "admin_dashboard_get",
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
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<Dashboard>)> {
    let dashboard = server_db.dashboard(&owner, &name).await?;
    let public = server_db
        .dashboard_public(dashboard.db_id.ok_or_else(|| dashboard_not_found(&name))?)
        .await?;
    Ok((
        StatusCode::OK,
        Json(dashboard_response(dashboard, DbUserRole::Admin, public)),
    ))
}

#[utoipa::path(post,
    path = "/api/v1/admin/dashboard/{owner}/{name}/add",
    operation_id = "admin_dashboard_add",
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
         (status = 404, description = "user not found"),
         (status = 409, description = "dashboard already exists"),
         (status = 413, description = "dashboard config too large"),
    )
)]
pub(crate) async fn add(
    _admin: AdminId,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
    Json(request): Json<DashboardConfig>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    let _ = server_db.user_id(&owner).await?;
    let commit_index = add_dashboard(&cluster, &server_db, owner, name, request.config).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(put,
    path = "/api/v1/admin/dashboard/{owner}/{name}/save",
    operation_id = "admin_dashboard_save",
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
         (status = 404, description = "dashboard not found"),
         (status = 413, description = "dashboard config too large"),
    )
)]
pub(crate) async fn save(
    _admin: AdminId,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
    Json(request): Json<DashboardConfig>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    validate_config(&request.config)?;
    let _ = server_db.dashboard(&owner, &name).await?;
    let commit_index = save_dashboard(&cluster, owner, name, request.config).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/admin/dashboard/{owner}/{name}/rename",
    operation_id = "admin_dashboard_rename",
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
         (status = 404, description = "dashboard not found"),
         (status = 409, description = "target dashboard already exists"),
    )
)]
pub(crate) async fn rename(
    _admin: AdminId,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
    request: Query<DashboardRenameParam>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    let _ = server_db.dashboard(&owner, &name).await?;
    let commit_index =
        rename_dashboard(&cluster, &server_db, owner, name, request.new_name.clone()).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(delete,
    path = "/api/v1/admin/dashboard/{owner}/{name}/remove",
    operation_id = "admin_dashboard_remove",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
    ),
    responses(
         (status = 204, description = "dashboard removed"),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "dashboard not found"),
    )
)]
pub(crate) async fn remove(
    _admin: AdminId,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    let _ = server_db.dashboard(&owner, &name).await?;
    let commit_index = remove_dashboard(&cluster, owner, name).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(put,
    path = "/api/v1/admin/dashboard/{owner}/{name}/public",
    operation_id = "admin_dashboard_public",
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
         (status = 404, description = "dashboard not found"),
    )
)]
pub(crate) async fn public(
    _admin: AdminId,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
    request: Query<DashboardPublicParam>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    let _ = server_db.dashboard(&owner, &name).await?;
    let commit_index = set_dashboard_public(&cluster, owner, name, request.public).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}
