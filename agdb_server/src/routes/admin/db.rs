pub(crate) mod user;

use crate::action::ClusterActionResult;
use crate::action::db_add::DbAdd;
use crate::action::db_backup::DbBackup;
use crate::action::db_clear::DbClear;
use crate::action::db_convert::DbConvert;
use crate::action::db_copy::DbCopy;
use crate::action::db_delete::DbDelete;
use crate::action::db_exec::DbExec;
use crate::action::db_optimize::DbOptimize;
use crate::action::db_remove::DbRemove;
use crate::action::db_rename::DbRename;
use crate::action::db_restore::DbRestore;
use crate::cluster::Cluster;
use crate::config::Config;
use crate::db_pool::DbPool;
use crate::error_code::ErrorCode;
use crate::routes::db::DbTypeParam;
use crate::routes::db::ServerDatabaseResource;
use crate::server_db::ServerDb;
use crate::server_error::ServerResponse;
use crate::server_error::permission_denied;
use crate::user_id::AdminId;
use crate::utilities::required_role;
use agdb_api::DbAudit;
use agdb_api::DbUserRole;
use agdb_api::Queries;
use agdb_api::QueriesResults;
use agdb_api::ServerDatabase;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Deserialize, IntoParams, ToSchema, agdb::TypeDefImpl)]
#[into_params(parameter_in = Query)]
pub struct ServerDatabaseAdminRename {
    pub new_owner: String,
    pub new_db: String,
}

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
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<DbTypeParam>,
) -> ServerResponse<impl IntoResponse> {
    let owner_id = server_db.user_id(&owner).await?;

    if server_db
        .find_user_db_id(owner_id, &owner, &db)
        .await?
        .is_some()
    {
        return Err(ErrorCode::DbExists.into());
    }

    let (commit_index, _result) = cluster
        .exec(DbAdd {
            owner: owner.to_string(),
            db,
            db_type: request.db_type,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
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
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<DbAudit>)> {
    let owner_id = server_db.user_id(&owner).await?;
    server_db.user_db_id(owner_id, &owner, &db).await?;

    Ok((StatusCode::OK, Json(db_pool.audit(&owner, &db).await?)))
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
         (status = 404, description = "db / user not found"),
    )
)]
pub(crate) async fn backup(
    _admin: AdminId,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<impl IntoResponse> {
    let owner_id = server_db.user_id(&owner).await?;
    server_db.user_db_id(owner_id, &owner, &db).await?;

    let (commit_index, _result) = cluster.exec(DbBackup { owner, db }).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/clear",
    operation_id = "admin_db_clear",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "user name"),
        ("db" = String, Path, description = "db name"),
        ServerDatabaseResource
    ),
    responses(
         (status = 201, description = "db resource(s) cleared", body = ServerDatabase),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn clear(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseResource>,
) -> ServerResponse<impl IntoResponse> {
    let owner_id = server_db.user_id(&owner).await?;
    let role = server_db.user_db_role(owner_id, &owner, &db).await?;

    let (commit_index, _result) = cluster
        .exec(DbClear {
            owner: owner.clone(),
            db: db.clone(),
            resource: request.resource,
        })
        .await?;

    let size = db_pool.db_size(&owner, &db).await.unwrap_or(0);
    let database = server_db.user_db(owner_id, &owner, &db).await?;
    let db = ServerDatabase {
        db,
        owner,
        db_type: database.db_type,
        role,
        backup: database.backup,
        size,
    };

    Ok((
        StatusCode::OK,
        [("commit-index", commit_index.to_string())],
        Json(db),
    ))
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
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn convert(
    _admin: AdminId,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<DbTypeParam>,
) -> ServerResponse<impl IntoResponse> {
    let owner_id = server_db.user_id(&owner).await?;
    let db_type = server_db.user_db(owner_id, &owner, &db).await?.db_type;

    if db_type == request.db_type {
        return Ok((StatusCode::CREATED, [("commit-index", String::new())]));
    }

    let (commit_index, _result) = cluster
        .exec(DbConvert {
            owner,
            db,
            db_type: request.db_type,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/copy",
    operation_id = "admin_db_copy",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ServerDatabaseAdminRename
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
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseAdminRename>,
) -> ServerResponse<impl IntoResponse> {
    let owner_id = server_db.user_id(&owner).await?;
    let db_type = server_db.user_db(owner_id, &owner, &db).await?.db_type;
    let new_owner_id = server_db.user_id(&request.new_owner).await?;

    if server_db
        .find_user_db_id(new_owner_id, &request.new_owner, &request.new_db)
        .await?
        .is_some()
    {
        return Err(ErrorCode::DbExists.into());
    }

    let (commit_index, _result) = cluster
        .exec(DbCopy {
            owner,
            db,
            new_owner: request.new_owner.clone(),
            new_db: request.new_db.clone(),
            db_type,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
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
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<impl IntoResponse> {
    let user_id = server_db.user_id(&owner).await?;
    let _ = server_db.user_db_id(user_id, &owner, &db).await?;

    let (commit_index, _result) = cluster.exec(DbDelete { owner, db }).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("commit-index", commit_index.to_string())],
    ))
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
         (status = 403, description = "mutable queries not allowed"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn exec(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
    Json(queries): Json<Queries>,
) -> ServerResponse<impl IntoResponse> {
    let required_role = required_role(&queries);
    if required_role != DbUserRole::Read {
        return Err(permission_denied(
            "mutable queries not allowed, use exec_mut endpoint",
        ));
    }
    let results = db_pool.exec(&owner, &db, queries).await?;
    Ok((StatusCode::OK, Json(QueriesResults(results))))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/exec_mut",
    operation_id = "admin_db_exec_mut",
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
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn exec_mut(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(cluster): State<Cluster>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    Json(queries): Json<Queries>,
) -> ServerResponse<impl IntoResponse> {
    let required_role = required_role(&queries);

    let (commit_index, results) = if required_role == DbUserRole::Read {
        (0, db_pool.exec(&owner, &db, queries).await?)
    } else {
        let mut index = 0;
        let mut results = Vec::new();

        if let (i, ClusterActionResult::QueryResults(r)) = cluster
            .exec(DbExec {
                user: config.admin.clone(),
                owner,
                db,
                queries,
            })
            .await?
        {
            index = i;
            results = r;
        }

        (index, results)
    };

    Ok((
        StatusCode::OK,
        [("commit-index", commit_index.to_string())],
        Json(QueriesResults(results)),
    ))
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
    State(server_db): State<ServerDb>,
) -> ServerResponse<(StatusCode, Json<Vec<ServerDatabase>>)> {
    let databases = server_db.dbs().await?;
    let mut dbs = Vec::with_capacity(databases.len());

    for db in databases {
        dbs.push(ServerDatabase {
            size: db_pool.db_size(&db.owner, &db.db).await.unwrap_or(0),
            db: db.db,
            owner: db.owner,
            db_type: db.db_type,
            role: DbUserRole::Admin,
            backup: db.backup,
        });
    }

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
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<impl IntoResponse> {
    let owner_id = server_db.user_id(&owner).await?;
    let database = server_db.user_db(owner_id, &owner, &db).await?;
    let role = server_db.user_db_role(owner_id, &owner, &db).await?;

    let (commit_index, _result) = cluster
        .exec(DbOptimize {
            owner: owner.clone(),
            db: db.clone(),
        })
        .await?;
    let size = db_pool.db_size(&owner, &db).await?;

    Ok((
        StatusCode::OK,
        [("commit-index", commit_index.to_string())],
        Json(ServerDatabase {
            db,
            owner,
            db_type: database.db_type,
            role,
            backup: database.backup,
            size,
        }),
    ))
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
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<impl IntoResponse> {
    let user_id = server_db.user_id(&owner).await?;
    let _ = server_db.user_db_id(user_id, &owner, &db).await?;

    let (commit_index, _result) = cluster.exec(DbRemove { owner, db }).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/rename",
    operation_id = "admin_db_rename",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ServerDatabaseAdminRename
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
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseAdminRename>,
) -> ServerResponse<impl IntoResponse> {
    let owner_id = server_db.user_id(&owner).await?;
    let _ = server_db.user_db_id(owner_id, &owner, &db).await?;

    if owner == request.new_owner && db == request.new_db {
        return Ok((StatusCode::CREATED, [("commit-index", String::new())]));
    }

    let new_owner_id = server_db.user_id(&request.new_owner).await?;
    if server_db
        .find_user_db_id(new_owner_id, &request.new_owner, &request.new_db)
        .await?
        .is_some()
    {
        return Err(ErrorCode::DbExists.into());
    }

    let (commit_index, _result) = cluster
        .exec(DbRename {
            owner,
            db,
            new_owner: request.new_owner.clone(),
            new_db: request.new_db.clone(),
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/admin/db/{owner}/{db}/restore",
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
         (status = 404, description = "db or backup not found"),
    )
)]
pub(crate) async fn restore(
    _admin: AdminId,
    State(cluster): State<Cluster>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<impl IntoResponse> {
    let owner_id = server_db.user_id(&owner).await?;
    server_db.user_db_id(owner_id, &owner, &db).await?;

    let (commit_index, _result) = cluster.exec(DbRestore { owner, db }).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}
