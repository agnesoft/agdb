use crate::db_pool::DbPool;
use crate::routes::db::user::DbUserRoleParam;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use agdb_api::DbUser;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(put,
    path = "/api/v1/admin/db/{owner}/{db}/user/{username}/add",
    operation_id = "admin_db_user_add",
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
    State(db_pool): State<DbPool>,
    Path((owner, db, username)): Path<(String, String, String)>,
    request: Query<DbUserRoleParam>,
) -> ServerResponse {
    let owner_id = db_pool.find_user_id(&owner)?;
    db_pool.add_db_user(&owner, &db, &username, request.0.db_role, owner_id)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/admin/db/{owner}/{db}/user/list",
    operation_id = "admin_db_user_list",
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
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<Vec<DbUser>>)> {
    let owner_id = db_pool.find_user_id(&owner)?;
    let users = db_pool.db_users(&owner, &db, owner_id)?;

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(delete,
    path = "/api/v1/admin/db/{owner}/{db}/user/{username}/remove",
    operation_id = "admin_db_user_remove",
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
    State(db_pool): State<DbPool>,
    Path((owner, db, username)): Path<(String, String, String)>,
) -> ServerResponse {
    let owner_id = db_pool.find_user_id(&owner)?;
    db_pool.remove_db_user(&owner, &db, &username, owner_id)?;

    Ok(StatusCode::NO_CONTENT)
}
