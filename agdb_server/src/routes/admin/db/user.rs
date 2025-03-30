use crate::action::db_user_add::DbUserAdd;
use crate::action::db_user_remove::DbUserRemove;
use crate::cluster::Cluster;
use crate::routes::db::user::DbUserRoleParam;
use crate::server_db::ServerDb;
use crate::server_error::ServerResponse;
use crate::server_error::permission_denied;
use crate::user_id::AdminId;
use agdb_api::DbUser;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[utoipa::path(put,
    path = "/api/v1/admin/db/{owner}/{db}/user/{username}/add",
    operation_id = "admin_db_user_add",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ("username" = String, Path, description = "user name"),
        DbUserRoleParam,
    ),
    responses(
         (status = 201, description = "user added"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "cannot change role of db owner"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn add(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path((owner, db, username)): Path<(String, String, String)>,
    request: Query<DbUserRoleParam>,
) -> ServerResponse<impl IntoResponse> {
    if owner == username {
        return Err(permission_denied("cannot change role of db owner"));
    }

    let owner_id = server_db.user_id(&owner).await?;
    let _ = server_db.user_db_id(owner_id, &owner, &db).await?;
    let _ = server_db.user_id(&username).await?;

    let (commit_index, _result) = cluster
        .exec(DbUserAdd {
            owner,
            db,
            user: username,
            db_role: request.db_role,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(get,
    path = "/api/v1/admin/db/{owner}/{db}/user/list",
    operation_id = "admin_db_user_list",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 200, description = "ok", body = Vec<DbUser>),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn list(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<Vec<DbUser>>)> {
    let owner_id = server_db.user_id(&owner).await?;
    let db_id = server_db.user_db_id(owner_id, &owner, &db).await?;

    Ok((StatusCode::OK, Json(server_db.db_users(db_id).await?)))
}

#[utoipa::path(delete,
    path = "/api/v1/admin/db/{owner}/{db}/user/{username}/remove",
    operation_id = "admin_db_user_remove",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ("username" = String, Path, description = "user name"),
    ),
    responses(
         (status = 204, description = "user removed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "cannot remove db owner"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn remove(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path((owner, db, username)): Path<(String, String, String)>,
) -> ServerResponse<impl IntoResponse> {
    if owner == username {
        return Err(permission_denied("cannot remove owner"));
    }

    let owner_id = server_db.user_id(&owner).await?;
    let _ = server_db.user_db_id(owner_id, &owner, &db).await?;
    let _ = server_db.user_id(&username).await?;

    let (commit_index, _result) = cluster
        .exec(DbUserRemove {
            owner,
            db,
            user: username,
        })
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("commit-index", commit_index.to_string())],
    ))
}
