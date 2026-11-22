use crate::action::dashboard_user_add::DashboardUserAdd;
use crate::action::dashboard_user_remove::DashboardUserRemove;
use crate::cluster::Cluster;
use crate::routes::db::user::DbUserRoleParam;
use crate::server_db::ServerDb;
use crate::server_db::dashboard_not_found;
use crate::server_error::CommitIndexHeader;
use crate::server_error::ServerResponse;
use crate::server_error::ServerResult;
use crate::server_error::permission_denied;
use crate::user_id::UserId;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;

/// Adds or changes the role of `username` (an existing user other than the
/// owner); returns the commit index.
pub(crate) async fn add_dashboard_user(
    cluster: &Cluster,
    server_db: &ServerDb,
    owner: String,
    name: String,
    username: String,
    role: DbUserRole,
) -> ServerResult<u64> {
    let _ = server_db.user_id(&username).await?;

    let (commit_index, _result) = cluster
        .exec(DashboardUserAdd {
            owner,
            name,
            user: username,
            role,
        })
        .await?;
    Ok(commit_index)
}

pub(crate) async fn remove_dashboard_user(
    cluster: &Cluster,
    owner: String,
    name: String,
    username: String,
) -> ServerResult<u64> {
    let (commit_index, _result) = cluster
        .exec(DashboardUserRemove {
            owner,
            name,
            user: username,
        })
        .await?;
    Ok(commit_index)
}

#[utoipa::path(put,
    path = "/api/v1/dashboard/{owner}/{name}/user/{username}/add",
    operation_id = "dashboard_user_add",
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
         (status = 403, description = "user must be a dashboard admin / cannot change role of dashboard owner"),
         (status = 404, description = "user / dashboard not found"),
    )
)]
pub(crate) async fn add(
    user: UserId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path((owner, name, username)): Path<(String, String, String)>,
    request: Query<DbUserRoleParam>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    // Users without access learn nothing about the dashboard (404) first.
    let access = server_db.user_dashboard(user.0, &owner, &name).await?;

    if access.role != DbUserRole::Admin {
        return Err(permission_denied("admin only"));
    }

    if owner == username {
        return Err(permission_denied("cannot change role of dashboard owner"));
    }

    let commit_index =
        add_dashboard_user(&cluster, &server_db, owner, name, username, request.db_role).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(get,
    path = "/api/v1/dashboard/{owner}/{name}/user/list",
    operation_id = "dashboard_user_list",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "dashboard owner user name"),
        ("name" = String, Path, description = "dashboard name"),
    ),
    responses(
         (status = 200, description = "ok", body = Vec<DbUser>),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "users of a public dashboard are listed to its users only"),
         (status = 404, description = "dashboard not found"),
    )
)]
pub(crate) async fn list(
    user: UserId,
    State(server_db): State<ServerDb>,
    Path((owner, name)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<Vec<DbUser>>)> {
    let access = server_db.user_dashboard(user.0, &owner, &name).await?;

    if !access.member {
        return Err(permission_denied("dashboard users only"));
    }

    let users = server_db
        .dashboard_users(
            access
                .dashboard
                .db_id
                .ok_or_else(|| dashboard_not_found(&name))?,
        )
        .await?;

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(delete,
    path = "/api/v1/dashboard/{owner}/{name}/user/{username}/remove",
    operation_id = "dashboard_user_remove",
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
         (status = 403, description = "must be admin / cannot remove dashboard owner"),
         (status = 404, description = "user / dashboard not found"),
    )
)]
pub(crate) async fn remove(
    user: UserId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path((owner, name, username)): Path<(String, String, String)>,
) -> ServerResponse<(StatusCode, CommitIndexHeader)> {
    // Users without access learn nothing about the dashboard (404) first.
    let access = server_db.user_dashboard(user.0, &owner, &name).await?;

    let user_id = server_db.user_id(&username).await?;

    if user.0 != user_id && access.role != DbUserRole::Admin {
        return Err(permission_denied("admin only"));
    }

    if owner == username {
        return Err(permission_denied("cannot remove owner"));
    }

    // A public viewer leaving has no role to remove: nothing to replicate.
    let commit_index = if user.0 == user_id && !access.member {
        0
    } else {
        remove_dashboard_user(&cluster, owner, name, username).await?
    };

    Ok((
        StatusCode::NO_CONTENT,
        [("commit-index", commit_index.to_string())],
    ))
}
