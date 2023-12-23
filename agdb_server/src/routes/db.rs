pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::db_backup_file;
use crate::db_pool::db_not_found;
use crate::db_pool::server_db::ServerDb;
use crate::db_pool::server_db_storage::ServerDbStorage;
use crate::db_pool::DbPool;
use crate::routes::db::user::DbUserRole;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use agdb::DbError;
use agdb::QueryError;
use agdb::QueryResult;
use agdb::QueryType;
use agdb::Transaction;
use agdb::TransactionMut;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::path::Path as FilePath;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Copy, Clone, Default, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DbType {
    #[default]
    Memory,
    Mapped,
    File,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub(crate) struct DbTypeParam {
    pub(crate) db_type: DbType,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ServerDatabaseSize {
    pub(crate) name: String,
    pub(crate) db_type: DbType,
    pub(crate) size: u64,
    pub(crate) backup: u64,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct ServerDatabaseWithRole {
    pub(crate) name: String,
    pub(crate) db_type: DbType,
    pub(crate) role: DbUserRole,
    pub(crate) size: u64,
    pub(crate) backup: u64,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub(crate) struct ServerDatabaseRename {
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

impl TryFrom<agdb::DbValue> for DbType {
    type Error = DbError;

    fn try_from(value: agdb::DbValue) -> Result<Self, Self::Error> {
        Ok(Self::from(value.to_string().as_str()))
    }
}

impl From<DbType> for agdb::DbValue {
    fn from(value: DbType) -> Self {
        value.to_string().into()
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
    path = "/api/v1/db/{owner}/{db}/add",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
        DbTypeParam,
    ),
    responses(
         (status = 201, description = "db added"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "cannot add db to another user"),
         (status = 465, description = "db already exists"),
         (status = 467, description = "db invalid"),
    )
)]
pub(crate) async fn add(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<DbTypeParam>,
) -> ServerResponse {
    let current_username = db_pool.user_name(user.0)?;

    if current_username != owner {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "cannot add db to another user",
        ));
    }

    db_pool.add_db(&owner, &db, request.db_type, &config)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/backup",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 201, description = "backup created"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "must be a db admin / memory db cannot have backup"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn backup(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let db_name = format!("{}/{}", owner, db);
    let mut database = db_pool.find_user_db(user.0, &db_name)?;

    if database.db_type == DbType::Memory {
        return Err(ServerError {
            description: "memory db cannot have backup".to_string(),
            status: StatusCode::FORBIDDEN,
        });
    }

    if !db_pool.is_db_admin(user.0, database.db_id.unwrap())? {
        return Err(ServerError {
            description: "must be a db admin".to_string(),
            status: StatusCode::FORBIDDEN,
        });
    }

    let pool = db_pool.get_pool()?;
    let server_db = pool.get(&db_name).ok_or(db_not_found(&db_name))?;
    let backup_path = db_backup_file(&config, &db_name);
    let backup_dir = backup_path.parent().unwrap();

    if backup_path.exists() {
        std::fs::remove_file(&backup_path)?;
    }

    std::fs::create_dir_all(backup_dir)?;

    server_db
        .get_mut()?
        .backup(backup_path.to_string_lossy().as_ref())?;
    database.backup = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    db_pool.save_db(database)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/delete",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 204, description = "db deleted"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db owner"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn delete(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let username = db_pool.user_name(user.0)?;

    if owner != username {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "user must be a db owner",
        ));
    }

    let db_name = format!("{}/{}", owner, db);
    let db = db_pool.find_user_db(user.0, &db_name)?;
    db_pool.delete_db(db, &config)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/exec",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
    ),
    request_body = Queries,
    responses(
         (status = 200, description = "ok", body = QueriesResults),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "must have at least write role"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn exec(
    user: UserId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
    Json(queries): Json<Queries>,
) -> ServerResponse<(StatusCode, Json<QueriesResults>)> {
    let db_name = format!("{}/{}", owner, db);
    let role = db_pool.find_user_db_role(user.0, &db_name)?;
    let required_role = required_role(&queries);

    if required_role == DbUserRole::Write && role == DbUserRole::Read {
        return Err(ServerError {
            description: "Permission denied: mutable queries require at least 'write' role (current role: 'read')".to_string(),
            status: StatusCode::FORBIDDEN,
        });
    }

    let pool = db_pool.get_pool()?;
    let db = pool.get(&db_name).ok_or(db_not_found(&db_name))?;

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
                db_type: db.db_type,
                role,
                size: pool
                    .get(&db.name)
                    .ok_or(db_not_found(&db.name))?
                    .get()?
                    .size(),
                backup: db.backup,
            })
        })
        .collect::<Result<Vec<ServerDatabaseWithRole>, ServerError>>()?;
    Ok((StatusCode::OK, Json(dbs)))
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/optimize",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 200, description = "ok", body = ServerDatabaseSize),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "must have write permissions"),
    )
)]
pub(crate) async fn optimize(
    user: UserId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<ServerDatabaseSize>)> {
    let db_name = format!("{owner}/{db}");
    let db = db_pool.find_user_db(user.0, &db_name)?;
    let role = db_pool.find_user_db_role(user.0, &db_name)?;

    if role == DbUserRole::Read {
        return Err(ServerError {
            description: "Permission denied: optimization can only be done with write permissions"
                .to_string(),
            status: StatusCode::FORBIDDEN,
        });
    }

    let pool = db_pool.get_pool()?;
    let server_db = pool.get(&db.name).ok_or(db_not_found(&db.name))?;
    server_db.get_mut()?.optimize_storage()?;
    let size = server_db.get()?.size();

    Ok((
        StatusCode::OK,
        Json(ServerDatabaseSize {
            name: db.name,
            db_type: db.db_type,
            size,
            backup: db.backup,
        }),
    ))
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/remove",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 204, description = "db removed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db owner"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn remove(
    user: UserId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let username = db_pool.user_name(user.0)?;

    if owner != username {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "user must be a db owner",
        ));
    }

    let db_name = format!("{}/{}", owner, db);
    let db = db_pool.find_user_db(user.0, &db_name)?;
    db_pool.remove_db(db)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/rename",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ServerDatabaseRename
    ),
    responses(
         (status = 201, description = "db renamed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db owner"),
         (status = 404, description = "user / db not found"),
         (status = 467, description = "invalid db"),
    )
)]
pub(crate) async fn rename(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseRename>,
) -> ServerResponse {
    let username = db_pool.user_name(user.0)?;

    if owner != username {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "user must be a db owner",
        ));
    }

    let db_name = format!("{}/{}", owner, db);
    db_pool.rename_db(&db_name, &request.new_name, &config)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/restore",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 201, description = "db restored"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "must be a db admin"),
         (status = 404, description = "backup not found"),
    )
)]
pub(crate) async fn restore(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let db_name = format!("{}/{}", owner, db);
    let mut database = db_pool.find_user_db(user.0, &db_name)?;

    if !db_pool.is_db_admin(user.0, database.db_id.unwrap())? {
        return Err(ServerError {
            description: "must be a db admin".to_string(),
            status: StatusCode::FORBIDDEN,
        });
    }

    let backup_path = db_backup_file(&config, &db_name);

    if !backup_path.exists() {
        return Err(ServerError {
            description: "backup not found".to_string(),
            status: StatusCode::NOT_FOUND,
        });
    }

    let current_path = FilePath::new(&config.data_dir).join(&db_name);
    let backup_temp = backup_path.parent().unwrap().join(db);

    db_pool.get_pool_mut()?.remove(&db_name);
    std::fs::rename(&current_path, &backup_temp)?;
    std::fs::rename(&backup_path, &current_path)?;
    std::fs::rename(backup_temp, backup_path)?;
    let server_db = ServerDb::new(&format!(
        "{}:{}",
        database.db_type,
        current_path.to_string_lossy()
    ))?;
    db_pool.get_pool_mut()?.insert(db_name, server_db);
    database.backup = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    db_pool.save_db(database)?;

    Ok(StatusCode::CREATED)
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
