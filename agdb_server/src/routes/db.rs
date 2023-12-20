pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::server_db_storage::ServerDbStorage;
use crate::db_pool::Database;
use crate::db_pool::DbPool;
use crate::error_code::ErrorCode;
use crate::routes::db::user::DbUserRole;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use agdb::QueryError;
use agdb::QueryResult;
use agdb::QueryType;
use agdb::Transaction;
use agdb::TransactionMut;
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

#[derive(Deserialize, ToSchema)]
pub(crate) struct ServerDatabase {
    pub(crate) name: String,
    pub(crate) db_type: DbType,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ServerDatabaseSize {
    pub(crate) name: String,
    pub(crate) db_type: DbType,
    pub(crate) size: u64,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ServerDatabaseWithRole {
    pub(crate) name: String,
    pub(crate) db_type: DbType,
    pub(crate) role: DbUserRole,
    pub(crate) size: u64,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub(crate) struct ServerDatabaseName {
    pub(crate) db: String,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct ServerDatabaseRename {
    pub(crate) db: String,
    pub(crate) new_name: String,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct Queries(pub(crate) Vec<QueryType>);

#[derive(Serialize, ToSchema)]
pub(crate) struct QueriesResults(pub(crate) Vec<QueryResult>);

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
         (status = 403, description = "user must be a db owner"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn delete(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Json(request): Json<ServerDatabaseName>,
) -> ServerResponse {
    let (db_user, _db) = request.db.split_once('/').ok_or(ErrorCode::DbInvalid)?;
    let name = db_pool.user_name(user.0)?;

    if db_user != name {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "user must be a db owner",
        ));
    }

    let db = db_pool.find_user_db(user.0, &request.db)?;
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
    let required_role = required_role(&queries);

    if required_role == DbUserRole::Write && role == DbUserRole::Read {
        return Err(ServerError {
            description: "Permission denied: mutable queries require at least 'write' role (current role: 'read')".to_string(),
            status: StatusCode::FORBIDDEN,
        });
    }

    let pool = db_pool.get_pool()?;
    let db = pool.get(&request.db).ok_or(ErrorCode::DbNotFound)?;

    let results = if required_role == DbUserRole::Read {
        db.get()?.transaction(|t| {
            let mut results = vec![];

            for q in &queries.0 {
                results.push(t_exec(t, q)?);
            }

            Ok(results)
        })
    } else {
        db.get_mut()?.transaction_mut(|t| {
            let mut results = vec![];

            for q in &queries.0 {
                results.push(t_exec_mut(t, q)?);
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
    let pool = db_pool.get_pool()?;
    let dbs = db_pool
        .find_user_dbs(user.0)?
        .into_iter()
        .map(|(db, role)| {
            Ok(ServerDatabaseWithRole {
                name: db.name.clone(),
                db_type: db.db_type.as_str().into(),
                role,
                size: pool
                    .get(&db.name)
                    .ok_or(ErrorCode::DbNotFound)?
                    .get()?
                    .size(),
            })
        })
        .collect::<Result<Vec<ServerDatabaseWithRole>, ServerError>>()?;
    Ok((StatusCode::OK, Json(dbs)))
}

#[utoipa::path(get,
    path = "/api/v1/db/optimize",
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = ServerDatabaseSize),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "permission denied / must have write permissions"),
    )
)]
pub(crate) async fn optimize(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<ServerDatabaseName>,
) -> ServerResponse<(StatusCode, Json<ServerDatabaseSize>)> {
    let db = db_pool.find_user_db(user.0, &request.db)?;
    let role = db_pool.find_user_db_role(user.0, &request.db)?;

    if role == DbUserRole::Read {
        return Err(ServerError {
            description: "Permission denied: optimization can only be done with write permissions"
                .to_string(),
            status: StatusCode::FORBIDDEN,
        });
    }

    let pool = db_pool.get_pool()?;
    let server_db = pool.get(&db.name).ok_or(ErrorCode::DbNotFound)?;
    server_db.get_mut()?.optimize_storage()?;
    let size = server_db.get()?.size();

    Ok((
        StatusCode::OK,
        Json(ServerDatabaseSize {
            name: db.name,
            db_type: db.db_type.as_str().into(),
            size,
        }),
    ))
}

#[utoipa::path(post,
    path = "/api/v1/db/remove",
    request_body = ServerDatabaseName,
    security(("Token" = [])),
    responses(
         (status = 204, description = "db removed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db owner"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn remove(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<ServerDatabaseName>,
) -> ServerResponse {
    let (db_user, _db) = request.db.split_once('/').ok_or(ErrorCode::DbInvalid)?;
    let name = db_pool.user_name(user.0)?;

    if db_user != name {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "user must be a db owner",
        ));
    }

    let db = db_pool.find_user_db(user.0, &request.db)?;
    db_pool.remove_db(db)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/db/rename",
    request_body = ServerDatabaseRename,
    security(("Token" = [])),
    responses(
         (status = 204, description = "db renamed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db admin"),
         (status = 466, description = "db not found"),
         (status = 467, description = "invalid db"),
    )
)]
pub(crate) async fn rename(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Json(request): Json<ServerDatabaseRename>,
) -> ServerResponse {
    let (db_user, _db) = request.db.split_once('/').ok_or(ErrorCode::DbInvalid)?;
    let (new_db_user, _db) = request
        .new_name
        .split_once('/')
        .ok_or(ErrorCode::DbInvalid)?;
    let db = db_pool.find_user_db(user.0, &request.db)?;

    if db_user != new_db_user {
        return Err(ErrorCode::DbInvalid.into());
    }

    if !db_pool.is_db_admin(user.0, db.db_id.unwrap())? {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "user must be a db admin",
        ));
    }

    db_pool.rename_db(db, &request.new_name, &config)?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) fn required_role(queries: &Queries) -> DbUserRole {
    for q in &queries.0 {
        match q {
            QueryType::InsertAlias(_)
            | QueryType::InsertEdges(_)
            | QueryType::InsertNodes(_)
            | QueryType::InsertValues(_)
            | QueryType::Remove(_)
            | QueryType::RemoveAliases(_)
            | QueryType::RemoveValues(_) => {
                return DbUserRole::Write;
            }
            _ => {}
        }
    }

    DbUserRole::Read
}

pub(crate) fn t_exec(
    t: &Transaction<ServerDbStorage>,
    q: &QueryType,
) -> Result<QueryResult, QueryError> {
    match q {
        QueryType::Search(q) => t.exec(q),
        QueryType::Select(q) => t.exec(q),
        QueryType::SelectAliases(q) => t.exec(q),
        QueryType::SelectAllAliases(q) => t.exec(q),
        QueryType::SelectKeys(q) => t.exec(q),
        QueryType::SelectKeyCount(q) => t.exec(q),
        QueryType::SelectValues(q) => t.exec(q),
        _ => unreachable!(),
    }
}

pub(crate) fn t_exec_mut(
    t: &mut TransactionMut<ServerDbStorage>,
    q: &QueryType,
) -> Result<QueryResult, QueryError> {
    match q {
        QueryType::Search(q) => t.exec(q),
        QueryType::Select(q) => t.exec(q),
        QueryType::SelectAliases(q) => t.exec(q),
        QueryType::SelectAllAliases(q) => t.exec(q),
        QueryType::SelectKeys(q) => t.exec(q),
        QueryType::SelectKeyCount(q) => t.exec(q),
        QueryType::SelectValues(q) => t.exec(q),
        QueryType::InsertAlias(q) => t.exec_mut(q),
        QueryType::InsertEdges(q) => t.exec_mut(q),
        QueryType::InsertNodes(q) => t.exec_mut(q),
        QueryType::InsertValues(q) => t.exec_mut(q),
        QueryType::Remove(q) => t.exec_mut(q),
        QueryType::RemoveAliases(q) => t.exec_mut(q),
        QueryType::RemoveValues(q) => t.exec_mut(q),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_pool::server_db::ServerDb;
    use agdb::QueryBuilder;

    #[test]
    #[should_panic]
    fn unreachable() {
        let db = ServerDb::new("memory:test").unwrap();
        db.get()
            .unwrap()
            .transaction(|t| t_exec(t, &QueryType::Remove(QueryBuilder::remove().ids(1).query())))
            .unwrap();
    }
}
