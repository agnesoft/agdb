use crate::db::DbPool;
use crate::server_error::ServerError;
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
         (status = 201, description = "User added"),
         (status = 461, description = "Database not found"),
         (status = 462, description = "User not found"),
         (status = 463, description = "Cannot add self"),
    )
)]
pub(crate) async fn add(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<DbUser>,
) -> Result<StatusCode, ServerError> {
    let db = db_pool.find_database(&request.database).map_err(|mut e| {
        e.status = StatusCode::from_u16(461).unwrap();
        e
    })?;
    let db_user = db_pool.find_user(&request.user).map_err(|mut e| {
        e.status = StatusCode::from_u16(462).unwrap();
        e
    })?;

    if db_user.db_id == Some(user.0) {
        return Ok(StatusCode::from_u16(463)?);
    }

    db_pool.add_database_user(
        db.db_id.unwrap(),
        db_user.db_id.unwrap(),
        &request.role.to_string(),
    )?;

    Ok(StatusCode::CREATED)
}
