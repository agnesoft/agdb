use crate::routes::db::user::DbUserRoleParam;
use crate::server_db::ServerDb;
use crate::server_error::permission_denied;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use crate::utilities::db_name;
use agdb_api::DbUser;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

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
    Path((owner, db, username)): Path<(String, String, String)>,
    request: Query<DbUserRoleParam>,
) -> ServerResponse {
    if owner == username {
        return Err(permission_denied("cannot change role of db owner"));
    }

    let db_name = db_name(&owner, &db);
    let owner_id = server_db.user_id(&owner).await?;
    let db_id = server_db.user_db_id(owner_id, &db_name).await?;
    let user_id = server_db.user_id(&username).await?;
    server_db
        .insert_db_user(db_id, user_id, request.db_role)
        .await?;

    Ok(StatusCode::CREATED)
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
    let db_id = server_db
        .user_db_id(owner_id, &db_name(&owner, &db))
        .await?;

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
    Path((owner, db, username)): Path<(String, String, String)>,
) -> ServerResponse {
    if owner == username {
        return Err(permission_denied("cannot remove owner"));
    }

    let owner_id = server_db.user_id(&owner).await?;
    let db_id = server_db
        .user_db_id(owner_id, &db_name(&owner, &db))
        .await?;
    let user_id = server_db.user_id(&username).await?;

    server_db.remove_db_user(db_id, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
