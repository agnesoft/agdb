use crate::db::DbPool;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DbUserRole {
    Admin,
    Write,
    Read,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct DbUser {
    pub(crate) database: String,
    pub(crate) user: String,
    pub(crate) role: DbUserRole,
}

impl Display for DbUserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbUserRole::Admin => f.write_str("admin"),
            DbUserRole::Write => f.write_str("write"),
            DbUserRole::Read => f.write_str("read"),
        }
    }
}

#[utoipa::path(post,
    path = "/api/v1/db/user/add",
    request_body = AddDatabaseUser,
    security(("Token" = [])),
    responses(
         (status = 201, description = "user added"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db admin / cannot add self"),
         (status = 464, description = "user not found"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn add(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<DbUser>,
) -> ServerResponse {
    let db = db_pool.find_database_id(&request.database)?;
    let db_user = db_pool.find_user_id(&request.user)?;

    if !db_pool.is_db_admin(user.0, db)? || db_user == user.0 {
        return Ok(StatusCode::FORBIDDEN);
    }

    db_pool.add_database_user(db, db_user, &request.role.to_string())?;

    Ok(StatusCode::CREATED)
}
