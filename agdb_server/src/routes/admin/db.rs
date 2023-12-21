pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::Database;
use crate::db_pool::DbPool;
use crate::error_code::ErrorCode;
use crate::routes::db::required_role;
use crate::routes::db::t_exec;
use crate::routes::db::t_exec_mut;
use crate::routes::db::user::DbUserRole;
use crate::routes::db::Queries;
use crate::routes::db::QueriesResults;
use crate::routes::db::ServerDatabase;
use crate::routes::db::ServerDatabaseName;
use crate::routes::db::ServerDatabaseSize;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use agdb::QueryError;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(post,
    path = "/api/v1/admin/db/add",
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
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Json(request): Json<ServerDatabase>,
) -> ServerResponse {
    let (db_user, _db) = request.name.split_once('/').ok_or(ErrorCode::DbInvalid)?;
    let db_user_id = db_pool.find_user_id(db_user)?;

    if db_pool.find_db_id(&request.name).is_ok() {
        return Err(ErrorCode::DbExists.into());
    }

    let db = Database {
        db_id: None,
        name: request.name,
        db_type: request.db_type.to_string(),
    };

    db_pool.add_db(db_user_id, db, &config)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/delete",
    request_body = ServerDatabaseName,
    security(("Token" = [])),
    responses(
         (status = 204, description = "db deleted"),
         (status = 401, description = "unauthorized"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn delete(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Json(request): Json<ServerDatabaseName>,
) -> ServerResponse {
    let db = db_pool.find_db(&request.db)?;
    db_pool.delete_db(db, &config)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/exec",
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
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    request: Query<ServerDatabaseName>,
    Json(queries): Json<Queries>,
) -> ServerResponse<(StatusCode, Json<QueriesResults>)> {
    let pool = db_pool.get_pool()?;
    let db = pool.get(&request.db).ok_or(ErrorCode::DbNotFound)?;
    let required_role = required_role(&queries);

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
    path = "/api/v1/admin/db/list",
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = Vec<ServerDatabaseSize>),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn list(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
) -> ServerResponse<(StatusCode, Json<Vec<ServerDatabaseSize>>)> {
    let pool = db_pool.get_pool()?;
    let dbs = db_pool
        .find_dbs()?
        .into_iter()
        .map(|db| {
            Ok(ServerDatabaseSize {
                name: db.name.clone(),
                db_type: db.db_type.as_str().into(),
                size: pool
                    .get(&db.name)
                    .ok_or(ErrorCode::DbNotFound)?
                    .get()?
                    .size(),
            })
        })
        .collect::<Result<Vec<ServerDatabaseSize>, ServerError>>()?;
    Ok((StatusCode::OK, Json(dbs)))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/remove",
    request_body = ServerDatabaseName,
    security(("Token" = [])),
    responses(
         (status = 204, description = "db removed"),
         (status = 401, description = "unauthorized"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn remove(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Json(request): Json<ServerDatabaseName>,
) -> ServerResponse {
    let db = db_pool.find_db(&request.db)?;
    db_pool.remove_db(db)?;

    Ok(StatusCode::NO_CONTENT)
}
