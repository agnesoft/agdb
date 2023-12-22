pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::db_not_found;
use crate::db_pool::Database;
use crate::db_pool::DbPool;
use crate::error_code::ErrorCode;
use crate::routes::db::required_role;
use crate::routes::db::t_exec;
use crate::routes::db::t_exec_mut;
use crate::routes::db::user::DbUserRole;
use crate::routes::db::DbTypeParam;
use crate::routes::db::Queries;
use crate::routes::db::QueriesResults;
use crate::routes::db::ServerDatabaseSize;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use agdb::QueryError;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(post,
    path = "/api/v1/admin/db/{username}/{db}/add",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
        DbTypeParam,
    ),
    responses(
         (status = 201, description = "db added"),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "user not found"),
         (status = 465, description = "db exists"),
    )
)]
pub(crate) async fn add(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((username, db)): Path<(String, String)>,
    request: Query<DbTypeParam>,
) -> ServerResponse {
    let user = db_pool.find_user_id(&username)?;
    let name = format!("{username}/{db}");

    if db_pool.find_user_db(user, &name).is_ok() {
        return Err(ErrorCode::DbExists.into());
    }

    let db = Database {
        db_id: None,
        name,
        db_type: request.db_type.to_string(),
    };

    db_pool.add_db(user, db, &config)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(delete,
    path = "/api/v1/admin/db/{owner}/{db}/delete",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 204, description = "db deleted"),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn delete(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let db_name = format!("{owner}/{db}");
    let db = db_pool.find_db(&db_name)?;
    db_pool.delete_db(db, &config)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/exec",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
    ),
    request_body = Queries,
    responses(
         (status = 200, description = "ok", body = QueriesResults),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "permission denied"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn exec(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
    Json(queries): Json<Queries>,
) -> ServerResponse<(StatusCode, Json<QueriesResults>)> {
    let pool = db_pool.get_pool()?;
    let db_name = format!("{}/{}", owner, db);
    let db = pool.get(&db_name).ok_or(db_not_found(&db_name))?;
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
                    .ok_or(db_not_found(&db.name))?
                    .get()?
                    .size(),
            })
        })
        .collect::<Result<Vec<ServerDatabaseSize>, ServerError>>()?;
    Ok((StatusCode::OK, Json(dbs)))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/optimize",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 200, description = "ok", body = ServerDatabaseSize),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn optimize(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<ServerDatabaseSize>)> {
    let db_name = format!("{owner}/{db}");
    let db = db_pool.find_db(&db_name)?;
    let pool = db_pool.get_pool()?;
    let server_db = pool.get(&db.name).ok_or(db_not_found(&db.name))?;
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

#[utoipa::path(delete,
    path = "/api/v1/admin/db/{owner}/{db}/remove",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 204, description = "db removed"),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn remove(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let name = format!("{owner}/{db}");
    let db = db_pool.find_db(&name)?;
    db_pool.remove_db(db)?;

    Ok(StatusCode::NO_CONTENT)
}
