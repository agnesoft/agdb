pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::Database;
use crate::db_pool::DbPool;
use crate::error_code::ErrorCode;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use agdb::QueryError;
use agdb::QueryResult;
use agdb::QueryType;
use axum::extract::Query;
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

#[derive(Serialize, ToSchema)]
pub(crate) struct ServerDatabaseWithRole {
    pub(crate) name: String,
    pub(crate) db_type: DbType,
    pub(crate) role: String,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub(crate) struct ServerDatabaseName {
    pub(crate) db: String,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct Queries(pub(crate) Vec<QueryType>);

#[derive(Serialize, ToSchema)]
pub(crate) struct QueriesResults(pub(crate) Vec<QueryResult>);

impl From<Database> for ServerDatabase {
    fn from(value: Database) -> Self {
        Self {
            name: value.name,
            db_type: value.db_type.as_str().into(),
        }
    }
}

impl From<&str> for DbType {
    fn from(value: &str) -> Self {
        match value {
            "mapped" => DbType::Mapped,
            "file" => DbType::File,
            _ => DbType::Memory,
        }
    }
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
    let db = db_pool.find_user_db(user.0, &request.db)?;

    if !db_pool.is_db_admin(user.0, db.db_id.unwrap())? {
        return Ok(StatusCode::FORBIDDEN);
    }

    db_pool.delete_db(db, &config)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/db/exec",
    request_body = Queries,
    params(
        ServerDatabaseName,
    ),
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = QueriesResults),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "permission denied"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn exec(
    user: UserId,
    State(db_pool): State<DbPool>,
    request: Query<ServerDatabaseName>,
    Json(queries): Json<Queries>,
) -> ServerResponse<(StatusCode, Json<QueriesResults>)> {
    let role = db_pool.find_user_db_role(user.0, &request.db)?;
    let mut required_role = "read";
    for q in &queries.0 {
        match q {
            QueryType::InsertAlias(_)
            | QueryType::InsertEdges(_)
            | QueryType::InsertNodes(_)
            | QueryType::InsertValues(_)
            | QueryType::Remove(_)
            | QueryType::RemoveAliases(_)
            | QueryType::RemoveValues(_) => {
                required_role = "write";
                break;
            }
            _ => {}
        }
    }

    if required_role == "write" && role == "read" {
        return Err(ServerError {
            description: "Permission denied: mutable queries require at least 'write' (current permission level: 'read')".to_string(),
            status: StatusCode::FORBIDDEN,
        });
    }

    let pool = db_pool.get_pool()?;
    let db = pool.get(&request.db).ok_or(ErrorCode::DbNotFound)?;

    let results = if required_role == "read" {
        db.get()?.transaction(|t| {
            let mut results = vec![];

            for q in &queries.0 {
                results.push(match q {
                    QueryType::Search(q) => t.exec(q)?,
                    QueryType::Select(q) => t.exec(q)?,
                    QueryType::SelectAliases(q) => t.exec(q)?,
                    QueryType::SelectAllAliases(q) => t.exec(q)?,
                    QueryType::SelectKeys(q) => t.exec(q)?,
                    QueryType::SelectKeyCount(q) => t.exec(q)?,
                    QueryType::SelectValues(q) => t.exec(q)?,
                    _ => unreachable!(),
                });
            }

            Ok(results)
        })
    } else {
        db.get_mut()?.transaction_mut(|t| {
            let mut results = vec![];

            for q in &queries.0 {
                results.push(match q {
                    QueryType::Search(q) => t.exec(q)?,
                    QueryType::Select(q) => t.exec(q)?,
                    QueryType::SelectAliases(q) => t.exec(q)?,
                    QueryType::SelectAllAliases(q) => t.exec(q)?,
                    QueryType::SelectKeys(q) => t.exec(q)?,
                    QueryType::SelectKeyCount(q) => t.exec(q)?,
                    QueryType::SelectValues(q) => t.exec(q)?,
                    QueryType::InsertAlias(q) => t.exec_mut(q)?,
                    QueryType::InsertEdges(q) => t.exec_mut(q)?,
                    QueryType::InsertNodes(q) => t.exec_mut(q)?,
                    QueryType::InsertValues(q) => t.exec_mut(q)?,
                    QueryType::Remove(q) => t.exec_mut(q)?,
                    QueryType::RemoveAliases(q) => t.exec_mut(q)?,
                    QueryType::RemoveValues(q) => t.exec_mut(q)?,
                });
            }

            Ok(results)
        })
    }
    .map_err(|e: QueryError| ServerError {
        description: e.to_string(),
        status: StatusCode::from_u16(470).unwrap(),
    })?;

    Ok((StatusCode::OK, Json(QueriesResults(results))))
}

#[utoipa::path(get,
    path = "/api/v1/db/list",
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = Vec<ServerDatabaseWithRole>),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn list(
    user: UserId,
    State(db_pool): State<DbPool>,
) -> ServerResponse<(StatusCode, Json<Vec<ServerDatabaseWithRole>>)> {
    let dbs = db_pool
        .find_user_dbs(user.0)?
        .into_iter()
        .map(|(db, role)| ServerDatabaseWithRole {
            name: db.name,
            db_type: db.db_type.as_str().into(),
            role,
        })
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
    let db = db_pool.find_user_db(user.0, &request.db)?;

    if !db_pool.is_db_admin(user.0, db.db_id.unwrap())? {
        return Ok(StatusCode::FORBIDDEN);
    }

    db_pool.remove_db(db)?;

    Ok(StatusCode::NO_CONTENT)
}
