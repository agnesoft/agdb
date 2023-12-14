pub(crate) mod user;

use crate::config::Config;
use crate::db::Database;
use crate::db::DbPool;
use crate::error_code::ErrorCode;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DbType {
    Memory,
    Mapped,
    File,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub(crate) struct ServerDatabase {
    pub(crate) name: String,
    pub(crate) db_type: DbType,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub(crate) struct ServerDatabaseName {
    pub(crate) name: String,
}

impl Display for DbType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbType::File => f.write_str("file"),
            DbType::Mapped => f.write_str("mapped"),
            DbType::Memory => f.write_str("memory"),
        }
    }
}

impl From<Database> for ServerDatabase {
    fn from(value: Database) -> Self {
        Self {
            name: value.name,
            db_type: match value.db_type.as_str() {
                "mapped" => DbType::Mapped,
                "file" => DbType::File,
                _ => DbType::Memory,
            },
        }
    }
}

#[utoipa::path(post,
    path = "/api/v1/db/add",
    request_body = ServerDatabase,
    security(("Token" = [])),
    responses(
         (status = 201, description = "db added"),
         (status = 401, description = "unauthorized"),
         (status = 465, description = "db already exists"),
         (status = 467, description = "db invalid"),
    )
)]
pub(crate) async fn add(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Json(request): Json<ServerDatabase>,
) -> ServerResponse {
    let (db_user, _db) = request.name.split_once('/').ok_or(ErrorCode::DbInvalid)?;
    let db_user_id = db_pool.find_user_id(db_user)?;

    if db_user_id != user.0 {
        return Err(ErrorCode::DbInvalid.into());
    }

    if db_pool.find_db_id(&request.name).is_ok() {
        return Err(ErrorCode::DbExists.into());
    }

    let db = Database {
        db_id: None,
        name: request.name,
        db_type: request.db_type.to_string(),
    };

    db_pool.add_db(user.0, db, &config)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/delete",
    request_body = ServerDatabaseName,
    security(("Token" = [])),
    responses(
         (status = 204, description = "db deleted"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db admin"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn delete(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Json(request): Json<ServerDatabaseName>,
) -> ServerResponse {
    let db = db_pool.find_user_db(user.0, &request.name)?;

    if !db_pool.is_db_admin(user.0, db.db_id.unwrap())? {
        return Ok(StatusCode::FORBIDDEN);
    }

    db_pool.delete_db(db, &config)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get,
    path = "/api/v1/db/list",
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = Vec<ServerDatabase>),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn list(
    user: UserId,
    State(db_pool): State<DbPool>,
) -> ServerResponse<(StatusCode, Json<Vec<ServerDatabase>>)> {
    let dbs = db_pool
        .find_user_dbs(user.0)?
        .into_iter()
        .map(|db| db.into())
        .collect();
    Ok((StatusCode::OK, Json(dbs)))
}

#[utoipa::path(post,
    path = "/api/v1/db/remove",
    request_body = ServerDatabaseName,
    security(("Token" = [])),
    responses(
         (status = 204, description = "db removed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db admin"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn remove(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<ServerDatabaseName>,
) -> ServerResponse {
    let db = db_pool.find_user_db(user.0, &request.name)?;

    if !db_pool.is_db_admin(user.0, db.db_id.unwrap())? {
        return Ok(StatusCode::FORBIDDEN);
    }

    db_pool.remove_db(db)?;

    Ok(StatusCode::NO_CONTENT)
}
