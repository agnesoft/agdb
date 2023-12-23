use crate::db_pool::DbPool;
use crate::routes::db::user::DbUser;
use crate::routes::db::user::DbUserRoleParam;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(put,
    path = "/api/v1/admin/db/{owner}/{db}/user/{username}/add",
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
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 200, description = "ok"),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn list(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<Vec<DbUser>>)> {
    let db_name = format!("{}/{}", owner, db);
    let db_id = db_pool.find_db_id(&db_name)?;
    let users = db_pool
        .db_users(db_id)?
        .into_iter()
        .map(|(user, role)| DbUser { user, role })
        .collect();

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(delete,
    path = "/api/v1/admin/db/{owner}/{db}/user/{username}/remove",
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
    if owner == username {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "cannot remove db owner",
        ));
    }

    let db_name = format!("{}/{}", owner, db);
    let db_id = db_pool.find_db_id(&db_name)?;
    let db_user = db_pool.db_user_id(db_id, &username)?;
    db_pool.remove_db_user(db_id, db_user)?;

    Ok(StatusCode::NO_CONTENT)
}
