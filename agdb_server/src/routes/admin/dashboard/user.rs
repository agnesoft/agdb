use crate::cluster::Cluster;
use crate::routes::dashboard::user::add_dashboard_user;
use crate::routes::dashboard::user::remove_dashboard_user;
use crate::routes::db::user::DbUserRoleParam;
use crate::server_db::ServerDb;
use crate::server_db::dashboard_not_found;
use crate::server_error::CommitIndexHeader;
use crate::server_error::ServerResponse;
use crate::server_error::permission_denied;
use crate::user_id::AdminId;
use agdb_api::DbUser;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;

#[utoipa::path(put,
    path = "/api/v1/admin/dashboard/{owner}/{name}/user/{username}/add",
    operation_id = "admin_dashboard_user_add",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
        ("username" = String, Path, description = "user name"),
        DbUserRoleParam,
    ),
    responses(
         (status = 201, description = "user added"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "cannot change role of dashboard owner"),
         (status = 404, description = "user / dashboard not found"),
    )
)]
pub(crate) async fn add(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path((owner, name, username)): Path<(String, String, String)>,
    request: Query<DbUserRoleParam>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    if owner == username {
        return Err(permission_denied("cannot change role of dashboard owner"));
    }

    let _ = server_db.dashboard(&owner, &name).await?;
    let commit_index =
        add_dashboard_user(&cluster, &server_db, owner, name, username, request.db_role).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(get,
    path = "/api/v1/admin/dashboard/{owner}/{name}/user/list",
    operation_id = "admin_dashboard_user_list",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
    ),
    responses(
         (status = 200, description = "ok", body = Vec<DbUser>),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "dashboard not found"),
    )
)]
pub(crate) async fn list(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<Vec<DbUser>>)> {
    let dashboard = server_db.dashboard(&owner, &name).await?;
    let users = server_db
        .dashboard_users(dashboard.db_id.ok_or_else(|| dashboard_not_found(&name))?)
        .await?;

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(delete,
    path = "/api/v1/admin/dashboard/{owner}/{name}/user/{username}/remove",
    operation_id = "admin_dashboard_user_remove",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
        ("username" = String, Path, description = "user name"),
    ),
    responses(
         (status = 204, description = "user removed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "cannot remove dashboard owner"),
         (status = 404, description = "user / dashboard not found"),
    )
)]
pub(crate) async fn remove(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path((owner, name, username)): Path<(String, String, String)>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    if owner == username {
        return Err(permission_denied("cannot remove owner"));
    }

    let _ = server_db.dashboard(&owner, &name).await?;
    let _ = server_db.user_id(&username).await?;
    let commit_index = remove_dashboard_user(&cluster, owner, name, username).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("commit-index", commit_index.to_string())],
    ))
}
