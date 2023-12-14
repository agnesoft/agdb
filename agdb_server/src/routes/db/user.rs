use crate::db::DbPool;
use crate::routes::db::ServerDatabaseName;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DbUserRole {
    Admin,
    Write,
    Read,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub(crate) struct DbUser {
    pub(crate) database: String,
    pub(crate) user: String,
    pub(crate) role: DbUserRole,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct RemoveDbUser {
    pub(crate) database: String,
    pub(crate) user: String,
}

impl Display for DbUserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbUserRole::Admin => f.write_str("admin"),
            DbUserRole::Write => f.write_str("write"),
            DbUserRole::Read => f.write_str("read"),
        }
    }
}

impl From<String> for DbUserRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "admin" => DbUserRole::Admin,
            "write" => DbUserRole::Write,
            _ => DbUserRole::Read,
        }
    }
}

#[utoipa::path(post,
    path = "/api/v1/db/user/add",
    request_body = AddDatabaseUser,
    security(("Token" = [])),
    responses(
         (status = 201, description = "user added"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db admin / cannot add self"),
         (status = 464, description = "user not found"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn add(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<DbUser>,
) -> ServerResponse {
    let db = db_pool.find_db_id(&request.database)?;
    let db_user = db_pool.find_user_id(&request.user)?;

    if !db_pool.is_db_admin(user.0, db)? || db_user == user.0 {
        return Ok(StatusCode::FORBIDDEN);
    }

    db_pool.add_db_user(db, db_user, &request.role.to_string())?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/db/user/list",
    security(("Token" = [])),
    params(
        ServerDatabaseName,
    ),
    responses(
         (status = 200, description = "ok"),
         (status = 401, description = "unauthorized"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn list(
    user: UserId,
    State(db_pool): State<DbPool>,
    request: Query<ServerDatabaseName>,
) -> ServerResponse<(StatusCode, Json<Vec<DbUser>>)> {
    let db = db_pool.find_user_db(user.0, &request.db)?;
    let users = db_pool
        .db_users(db.db_id.unwrap())?
        .into_iter()
        .map(|(name, role)| DbUser {
            database: request.db.clone(),
            user: name,
            role: role.into(),
        })
        .collect();

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(post,
    path = "/api/v1/db/user/remove",
    request_body = RemoveDbUser,
    security(("Token" = [])),
    responses(
         (status = 204, description = "user removed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db admin / cannot remove last admin user"),
         (status = 464, description = "user not found"),
         (status = 466, description = "db not found"),
    )
)]
pub(crate) async fn remove(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<RemoveDbUser>,
) -> ServerResponse {
    let db = db_pool.find_db_id(&request.database)?;
    let db_user = db_pool.find_user_id(&request.user)?;
    let admins = db_pool.db_admins(db)?;

    println!("{:?} == {:?}", admins, vec![db_user]);

    if (!admins.contains(&user.0) && user.0 != db_user) || admins == vec![db_user] {
        return Ok(StatusCode::FORBIDDEN);
    }

    db_pool.remove_db_user(db, db_user)?;

    Ok(StatusCode::NO_CONTENT)
}
