pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::DbPool;
use crate::routes::db::DbTypeParam;
use crate::routes::db::ServerDatabaseRename;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use agdb_api::Queries;
use agdb_api::QueriesResults;
use agdb_api::ServerDatabase;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/add",
    operation_id = "admin_db_add",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
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
    Path((owner, db)): Path<(String, String)>,
    request: Query<DbTypeParam>,
) -> ServerResponse {
    db_pool.add_db(&owner, &db, request.db_type, &config)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/backup",
    operation_id = "admin_db_backup",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 201, description = "backup created"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "memory db cannot have backup"),
         (status = 404, description = "db / user not found"),
    )
)]
pub(crate) async fn backup(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let owner_id = db_pool.find_user_id(&owner)?;
    db_pool.backup_db(&owner, &db, owner_id, &config)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/copy",
    operation_id = "admin_db_copy",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ServerDatabaseRename
    ),
    responses(
         (status = 201, description = "db copied"),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "user / db not found"),
         (status = 465, description = "target db exists"),
         (status = 467, description = "invalid db"),
    )
)]
pub(crate) async fn copy(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseRename>,
) -> ServerResponse {
    let owner_id = db_pool.find_user_id(&owner)?;
    db_pool.copy_db(&owner, &db, &request.new_name, owner_id, &config, true)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(delete,
    path = "/api/v1/admin/db/{owner}/{db}/delete",
    operation_id = "admin_db_delete",
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
    let owner_id = db_pool.find_user_id(&owner)?;
    db_pool.delete_db(&owner, &db, owner_id, &config)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/exec",
    operation_id = "admin_db_exec",
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
    let owner_id = db_pool.find_user_id(&owner)?;
    let results = db_pool.exec(&owner, &db, owner_id, &queries)?;

    Ok((StatusCode::OK, Json(QueriesResults(results))))
}

#[utoipa::path(get,
    path = "/api/v1/admin/db/list",
    operation_id = "admin_db_list",
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = Vec<ServerDatabase>),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn list(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
) -> ServerResponse<(StatusCode, Json<Vec<ServerDatabase>>)> {
    let dbs = db_pool.find_dbs()?;

    Ok((StatusCode::OK, Json(dbs)))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/optimize",
    operation_id = "admin_db_optimize",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 200, description = "ok", body = ServerDatabase),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn optimize(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<ServerDatabase>)> {
    let owner_id = db_pool.find_user_id(&owner)?;
    let db = db_pool.optimize_db(&owner, &db, owner_id)?;

    Ok((StatusCode::OK, Json(db)))
}

#[utoipa::path(delete,
    path = "/api/v1/admin/db/{owner}/{db}/remove",
    operation_id = "admin_db_remove",
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
    let owner_id = db_pool.find_user_id(&owner)?;
    db_pool.remove_db(&owner, &db, owner_id)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/rename",
    operation_id = "admin_db_rename",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ServerDatabaseRename
    ),
    responses(
         (status = 201, description = "db renamed"),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "user / db not found"),
         (status = 465, description = "target db exists"),
         (status = 467, description = "invalid db"),
    )
)]
pub(crate) async fn rename(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseRename>,
) -> ServerResponse {
    let owner_id = db_pool.find_user_id(&owner)?;
    db_pool.rename_db(&owner, &db, &request.new_name, owner_id, &config)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/admin/{owner}/{db}/restore",
    operation_id = "admin_db_restore",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 201, description = "db restored"),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "backup not found"),
    )
)]
pub(crate) async fn restore(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let owner_id = db_pool.find_user_id(&owner)?;
    db_pool.restore_db(&owner, &db, owner_id, &config)?;

    Ok(StatusCode::CREATED)
}
