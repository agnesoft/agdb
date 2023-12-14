use crate::db::DbPool;
use crate::routes::db::user::DbUser;
use crate::routes::db::ServerDatabaseName;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

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
            role: role.into(),
        })
        .collect();

    Ok((StatusCode::OK, Json(users)))
}
