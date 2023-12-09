pub(crate) mod user;

use crate::db::Database;
use crate::db::DbPool;
use crate::error_code::ErrorCode;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
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

#[derive(Deserialize, ToSchema)]
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
         (status = 201, description = "database added"),
         (status = 401, description = "unauthorized"),
         (status = 465, description = "db already exists"),
         (status = 467, description = "db invalid"),
    )
)]
pub(crate) async fn add(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<ServerDatabase>,
) -> Result<StatusCode, ServerError> {
    if db_pool.find_database_id(&request.name).is_ok() {
        return Ok(ErrorCode::DbExists.into());
    }
    let db = Database {
        db_id: None,
        name: request.name,
        db_type: request.db_type.to_string(),
    };
    db_pool
        .add_database(user.0, db)
        .map_err(|mut e: ServerError| {
            e.status = ErrorCode::DbNameInvalid.into();
            e.description = format!("{}: {}", ErrorCode::DbNameInvalid.as_str(), e.description);
            e
        })?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/delete",
    request_body = ServerDatabaseName,
    security(("Token" = [])),
    responses(
         (status = 204, description = "database deleted"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn delete(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<ServerDatabaseName>,
) -> ServerResponse {
    let db = db_pool.find_user_database(user.0, &request.name)?;
    db_pool.delete_database(db)?;

    Ok(StatusCode::OK)
}

#[utoipa::path(get,
    path = "/api/v1/db/list",
    security(("Token" = [])),
    responses(
         (status = 200, description = "Ok", body = Vec<ServerDatabase>)
    )
)]
pub(crate) async fn list(
    user: UserId,
    State(db_pool): State<DbPool>,
) -> Result<(StatusCode, Json<Vec<ServerDatabase>>), ServerError> {
    let dbs = db_pool
        .find_user_databases(user.0)?
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
         (status = 200, description = "Database removed"),
         (status = 403, description = "Database not found for user"),
    )
)]
pub(crate) async fn remove(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<ServerDatabaseName>,
) -> Result<StatusCode, ServerError> {
    let db = db_pool
        .find_user_database(user.0, &request.name)
        .map_err(|mut e| {
            e.status = StatusCode::FORBIDDEN;
            e
        })?;

    db_pool.remove_database(db)?;

    Ok(StatusCode::OK)
}
