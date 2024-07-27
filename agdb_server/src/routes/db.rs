pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::DbPool;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use agdb_api::DbAudit;
use agdb_api::DbResource;
use agdb_api::DbType;
use agdb_api::Queries;
use agdb_api::QueriesResults;
use agdb_api::ServerDatabase;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ServerDatabaseRename {
    pub new_name: String,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub(crate) struct DbTypeParam {
    pub(crate) db_type: DbType,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ServerDatabaseResource {
    pub resource: DbResource,
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/add",
    operation_id = "db_add",
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
    let current_username = db_pool.user_name(user.0).await?;

    if current_username != owner {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "cannot add db to another user",
        ));
    }

    db_pool
        .add_db(&owner, &db, request.db_type, &config)
        .await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/db/{owner}/{db}/audit",
    operation_id = "db_audit",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name")
    ),
    responses(
         (status = 200, description = "ok", body = DbAudit),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn audit(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<DbAudit>)> {
    let results = db_pool.audit(&owner, &db, user.0, &config).await?;

    Ok((StatusCode::OK, Json(results)))
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/backup",
    operation_id = "db_backup",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 201, description = "backup created"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "must be a db admin / memory db cannot have backup"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn backup(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    db_pool.backup_db(&owner, &db, user.0, &config).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/clear",
    operation_id = "db_clear",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
        ServerDatabaseResource
    ),
    responses(
         (status = 201, description = "db resource(s) cleared", body = ServerDatabase),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "must be a db admin"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn clear(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseResource>,
) -> ServerResponse<(StatusCode, Json<ServerDatabase>)> {
    let db = db_pool
        .clear_db(&owner, &db, user.0, &config, request.resource)
        .await?;

    Ok((StatusCode::OK, Json(db)))
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/copy",
    operation_id = "db_copy",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ServerDatabaseRename
    ),
    responses(
         (status = 201, description = "db copied"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "cannot copy db to another user"),
         (status = 404, description = "user / db not found"),
         (status = 465, description = "target db exists"),
         (status = 467, description = "invalid db"),
    )
)]
pub(crate) async fn copy(
    user: UserId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseRename>,
) -> ServerResponse {
    db_pool
        .copy_db(&owner, &db, &request.new_name, user.0, &config, false)
        .await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/delete",
    operation_id = "db_delete",
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
    db_pool.delete_db(&owner, &db, user.0, &config).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/exec",
    operation_id = "db_exec",
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
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    Json(queries): Json<Queries>,
) -> ServerResponse<(StatusCode, Json<QueriesResults>)> {
    let results = db_pool.exec(&owner, &db, user.0, queries, &config).await?;

    Ok((StatusCode::OK, Json(QueriesResults(results))))
}

#[utoipa::path(get,
    path = "/api/v1/db/list",
    operation_id = "db_list",
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
    let dbs = db_pool.find_user_dbs(user.0).await?;

    Ok((StatusCode::OK, Json(dbs)))
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/optimize",
    operation_id = "db_optimize",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 200, description = "ok", body = ServerDatabase),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "must have write permissions"),
    )
)]
pub(crate) async fn optimize(
    user: UserId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<ServerDatabase>)> {
    let db = db_pool.optimize_db(&owner, &db, user.0).await?;

    Ok((StatusCode::OK, Json(db)))
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/remove",
    operation_id = "db_remove",
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
    db_pool.remove_db(&owner, &db, user.0).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/rename",
    operation_id = "db_rename",
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
         (status = 465, description = "target db exists"),
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
    db_pool
        .rename_db(&owner, &db, &request.new_name, user.0, &config)
        .await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/restore",
    operation_id = "db_restore",
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
    db_pool.restore_db(&owner, &db, user.0, &config).await?;

    Ok(StatusCode::CREATED)
}
