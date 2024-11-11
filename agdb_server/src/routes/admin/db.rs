pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::DbPool;
use crate::error_code::ErrorCode;
use crate::routes::db::DbTypeParam;
use crate::routes::db::ServerDatabaseRename;
use crate::server_db::Database;
use crate::server_db::ServerDb;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use crate::utilities::db_name;
use agdb_api::DbAudit;
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
    tag = "agdb",
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
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<DbTypeParam>,
) -> ServerResponse {
    let name = db_name(&owner, &db);
    let user = server_db.user_id(&owner).await?;

    if server_db.find_user_db_id(user, &name).await?.is_some() {
        return Err(ErrorCode::DbExists.into());
    }

    let backup = db_pool
        .add_db(&owner, &name, request.db_type, &config)
        .await?;

    server_db
        .insert_db(
            user,
            Database {
                db_id: None,
                name,
                db_type: request.db_type,
                backup,
            },
        )
        .await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/admin/db/{owner}/{db}/audit",
    operation_id = "admin_db_audit",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name")
    ),
    responses(
         (status = 200, description = "ok", body = DbAudit),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn audit(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<DbAudit>)> {
    let owner_id = db_pool.find_user_id(&owner).await?;
    let results = db_pool.audit(&owner, &db, owner_id, &config).await?;

    Ok((StatusCode::OK, Json(results)))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/backup",
    operation_id = "admin_db_backup",
    tag = "agdb",
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
    let owner_id = db_pool.find_user_id(&owner).await?;
    db_pool.backup_db(&owner, &db, owner_id, &config).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/convert",
    operation_id = "admin_db_convert",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
        DbTypeParam,
    ),
    responses(
         (status = 201, description = "db typ changes"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "admin only"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn convert(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<DbTypeParam>,
) -> ServerResponse {
    let owner_id = db_pool.find_user_id(&owner).await?;
    db_pool
        .convert_db(&owner, &db, owner_id, request.db_type, &config)
        .await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/copy",
    operation_id = "admin_db_copy",
    tag = "agdb",
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
    let owner_id = db_pool.find_user_id(&owner).await?;
    db_pool
        .copy_db(&owner, &db, &request.new_name, owner_id, &config, true)
        .await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(delete,
    path = "/api/v1/admin/db/{owner}/{db}/delete",
    operation_id = "admin_db_delete",
    tag = "agdb",
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
    let owner_id = db_pool.find_user_id(&owner).await?;
    db_pool.delete_db(&owner, &db, owner_id, &config).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/exec",
    operation_id = "admin_db_exec",
    tag = "agdb",
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
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    Json(queries): Json<Queries>,
) -> ServerResponse<(StatusCode, Json<QueriesResults>)> {
    let owner_id = db_pool.find_user_id(&owner).await?;
    let results = db_pool
        .exec(&owner, &db, owner_id, queries, &config)
        .await?;

    Ok((StatusCode::OK, Json(QueriesResults(results))))
}

#[utoipa::path(get,
    path = "/api/v1/admin/db/list",
    operation_id = "admin_db_list",
    tag = "agdb",
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
    let dbs = db_pool.find_dbs().await?;

    Ok((StatusCode::OK, Json(dbs)))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/optimize",
    operation_id = "admin_db_optimize",
    tag = "agdb",
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
    let owner_id = db_pool.find_user_id(&owner).await?;
    let db = db_pool.optimize_db(&owner, &db, owner_id).await?;

    Ok((StatusCode::OK, Json(db)))
}

#[utoipa::path(delete,
    path = "/api/v1/admin/db/{owner}/{db}/remove",
    operation_id = "admin_db_remove",
    tag = "agdb",
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
    let owner_id = db_pool.find_user_id(&owner).await?;
    db_pool.remove_db(&owner, &db, owner_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/rename",
    operation_id = "admin_db_rename",
    tag = "agdb",
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
    let owner_id = db_pool.find_user_id(&owner).await?;
    db_pool
        .rename_db(&owner, &db, &request.new_name, owner_id, &config)
        .await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/admin/{owner}/{db}/restore",
    operation_id = "admin_db_restore",
    tag = "agdb",
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
    let owner_id = db_pool.find_user_id(&owner).await?;
    db_pool.restore_db(&owner, &db, owner_id, &config).await?;

    Ok(StatusCode::CREATED)
}
