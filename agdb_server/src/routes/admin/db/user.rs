use crate::db_pool::DbPool;
use crate::error_code::ErrorCode;
use crate::routes::db::user::DbUser;
use crate::routes::db::user::RemoveDbUser;
use crate::routes::db::ServerDatabaseName;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(put,
    path = "/api/v1/admin/db/user/add",
    request_body = AddDatabaseUser,
    security(("Token" = [])),
    responses(
         (status = 201, description = "user added"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "cannot change role of db owner"),
         (status = 464, description = "user not found"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn add(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Json(request): Json<DbUser>,
) -> ServerResponse {
    let (db_user, _db) = request
        .database
        .split_once('/')
        .ok_or(ErrorCode::DbInvalid)?;

    if db_user == request.user {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "cannot change role of db owner",
        ));
    }

    let db = db_pool.find_db_id(&request.database)?;
    let db_user = db_pool.find_user_id(&request.user)?;
    db_pool.add_db_user(db, db_user, request.role)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/admin/db/user/list",
    security(("Token" = [])),
    params(
        ServerDatabaseName,
    ),
    responses(
         (status = 200, description = "ok"),
         (status = 401, description = "unauthorized"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn list(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    request: Query<ServerDatabaseName>,
) -> ServerResponse<(StatusCode, Json<Vec<DbUser>>)> {
    let db = db_pool.find_db_id(&request.db)?;
    let users = db_pool
        .db_users(db)?
        .into_iter()
        .map(|(name, role)| DbUser {
            database: request.db.clone(),
            user: name,
            role,
        })
        .collect();

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/user/remove",
    request_body = RemoveDbUser,
    security(("Token" = [])),
    responses(
         (status = 204, description = "user removed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "cannot remove db owner"),
         (status = 464, description = "user not found"),
         (status = 466, description = "db not found"),
         (status = 467, description = "db invalid"),
    )
)]
pub(crate) async fn remove(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Json(request): Json<RemoveDbUser>,
) -> ServerResponse {
    let (db_user, _db) = request
        .database
        .split_once('/')
        .ok_or(ErrorCode::DbInvalid)?;

    if db_user == request.user {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "cannot remove db owner",
        ));
    }

    let db = db_pool.find_db_id(&request.database)?;
    let db_user = db_pool.db_user_id(db, &request.user)?;
    db_pool.remove_db_user(db, db_user)?;

    Ok(StatusCode::NO_CONTENT)
}
