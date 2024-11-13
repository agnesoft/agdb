pub(crate) mod user;

use crate::config::Config;
use crate::db_pool::DbPool;
use crate::error_code::ErrorCode;
use crate::routes::db::DbTypeParam;
use crate::routes::db::ServerDatabaseRename;
use crate::routes::db::ServerDatabaseResource;
use crate::server_db::Database;
use crate::server_db::ServerDb;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use crate::utilities::db_name;
use crate::utilities::required_role;
use agdb_api::DbAudit;
use agdb_api::DbUserRole;
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
    let owner_id = server_db.user_id(&owner).await?;

    if server_db.find_user_db_id(owner_id, &name).await?.is_some() {
        return Err(ErrorCode::DbExists.into());
    }

    let backup = db_pool
        .add_db(&owner, &db, &name, request.db_type, &config)
        .await?;

    server_db
        .insert_db(
            owner_id,
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
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<DbAudit>)> {
    let db_name = db_name(&owner, &db);
    let owner_id = server_db.user_id(&owner).await?;
    server_db.user_db_id(owner_id, &db_name).await?;

    Ok((
        StatusCode::OK,
        Json(db_pool.audit(&owner, &db, &config).await?),
    ))
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
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let db_name = db_name(&owner, &db);
    let owner_id = server_db.user_id(&owner).await?;
    let mut database = server_db.user_db(owner_id, &db_name).await?;

    database.backup = db_pool
        .backup_db(&owner, &db, &db_name, database.db_type, &config)
        .await?;

    server_db.save_db(&database).await?;

    Ok(StatusCode::CREATED)
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
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseResource>,
) -> ServerResponse<(StatusCode, Json<ServerDatabase>)> {
    let db_name = db_name(&owner, &db);
    let owner_id = server_db.user_id(&owner).await?;
    let mut database = server_db.user_db(owner_id, &db_name).await?;
    let role = server_db.user_db_role(owner_id, &db_name).await?;
    let db = db_pool
        .clear_db(&owner, &db, &mut database, role, &config, request.resource)
        .await?;
    server_db.save_db(&database).await?;

    Ok((StatusCode::OK, Json(db)))
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
         (status = 403, description = "server admin only"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn convert(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<DbTypeParam>,
) -> ServerResponse {
    let db_name = db_name(&owner, &db);
    let owner_id = server_db.user_id(&owner).await?;
    let mut database = server_db.user_db(owner_id, &db_name).await?;

    if database.db_type == request.db_type {
        return Ok(StatusCode::CREATED);
    }

    db_pool
        .convert_db(
            &owner,
            &db,
            &db_name,
            database.db_type,
            request.db_type,
            &config,
        )
        .await?;

    database.db_type = request.db_type;
    server_db.save_db(&database).await?;

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
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseRename>,
) -> ServerResponse {
    let (new_owner, new_db) = request
        .new_name
        .split_once('/')
        .ok_or(ErrorCode::DbInvalid)?;
    let source_db = db_name(&owner, &db);
    let target_db = db_name(new_owner, new_db);
    let owner_id = server_db.user_id(&owner).await?;
    let db_type = server_db.user_db(owner_id, &source_db).await?.db_type;
    let new_owner_id = server_db.user_id(new_owner).await?;

    if server_db
        .find_user_db_id(new_owner_id, &target_db)
        .await?
        .is_some()
    {
        return Err(ErrorCode::DbExists.into());
    }

    db_pool
        .copy_db(&source_db, new_owner, new_db, &target_db, &config)
        .await?;

    server_db
        .save_db(&Database {
            db_id: None,
            name: target_db,
            db_type,
            backup: 0,
        })
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
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let owner_id = server_db.user_id(&owner).await?;
    let db_name = db_name(&owner, &db);
    server_db.remove_db(owner_id, &db_name).await?;
    db_pool.delete_db(&owner, &db, &db_name, &config).await?;

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
    let db_name = db_name(&owner, &db);
    let required_role = required_role(&queries);

    let results = if required_role == DbUserRole::Read {
        db_pool.exec(&db_name, queries).await?
    } else {
        db_pool
            .exec_mut(&owner, &db, &db_name, &config.admin, queries, &config)
            .await?
    };

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
    State(server_db): State<ServerDb>,
) -> ServerResponse<(StatusCode, Json<Vec<ServerDatabase>>)> {
    let databases = server_db.dbs().await?;
    let mut dbs = Vec::with_capacity(databases.len());

    for db in databases {
        dbs.push(ServerDatabase {
            size: db_pool.db_size(&db.name).await.unwrap_or(0),
            name: db.name,
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
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<ServerDatabase>)> {
    let db_name = db_name(&owner, &db);
    let owner_id = server_db.user_id(&owner).await?;
    let database = server_db.user_db(owner_id, &db_name).await?;
    let role = server_db.user_db_role(owner_id, &db_name).await?;
    let size = db_pool.optimize_db(&db_name).await?;

    Ok((
        StatusCode::OK,
        Json(ServerDatabase {
            name: db_name,
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
    State(db_pool): State<DbPool>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let db_name = db_name(&owner, &db);
    let owner_id = server_db.user_id(&owner).await?;
    server_db.remove_db(owner_id, &db_name).await?;
    db_pool.remove_db(&db_name).await?;

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
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
    request: Query<ServerDatabaseRename>,
) -> ServerResponse {
    let db_name = db_name(&owner, &db);

    if db_name == request.new_name {
        return Ok(StatusCode::CREATED);
    }

    let (new_owner, new_db) = request
        .new_name
        .split_once('/')
        .ok_or(ErrorCode::DbInvalid)?;
    let new_owner_id = server_db.user_id(new_owner).await?;
    let owner_id = server_db.user_id(&owner).await?;
    let mut database = server_db.user_db(owner_id, &db_name).await?;

    db_pool
        .rename_db(
            &owner,
            &db,
            &db_name,
            new_owner,
            new_db,
            &request.new_name,
            &config,
        )
        .await?;

    database.name = request.new_name.clone();
    server_db.save_db(&database).await?;

    if new_owner != owner {
        server_db
            .insert_db_user(database.db_id.unwrap(), new_owner_id, DbUserRole::Admin)
            .await?;
    }

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
    State(server_db): State<ServerDb>,
    State(config): State<Config>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse {
    let db_name = db_name(&owner, &db);
    let owner_id = server_db.user_id(&owner).await?;
    let mut database = server_db.user_db(owner_id, &db_name).await?;

    if let Some(backup) = db_pool
        .restore_db(&owner, &db, &db_name, database.db_type, &config)
        .await?
    {
        database.backup = backup;
        server_db.save_db(&database).await?;
    }

    Ok(StatusCode::CREATED)
}
