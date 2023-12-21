use crate::db_pool::DbPool;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DbUserRole {
    Admin,
    Write,
    Read,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub(crate) struct DbUserRoleParam {
    pub(crate) db_role: DbUserRole,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct DbUser {
    pub(crate) user: String,
    pub(crate) role: DbUserRole,
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/user/{username}/add",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ("username" = String, Path, description = "user name"),
        DbUserRoleParam,
    ),
    responses(
         (status = 201, description = "user added"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "user must be a db admin / cannot change role of db owner"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn add(
    user: UserId,
    State(db_pool): State<DbPool>,
    Path((owner, db, username)): Path<(String, String, String)>,
    request: Query<DbUserRoleParam>,
) -> ServerResponse {
    if owner == username {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "cannot change role of db owner",
        ));
    }

    let db_name = format!("{}/{}", owner, db);
    let db_id = db_pool.find_db_id(&db_name)?;

    if !db_pool.is_db_admin(user.0, db_id)? {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "must be a db admin",
        ));
    }

    let db_user = db_pool.find_user_id(&username)?;
    db_pool.add_db_user(db_id, db_user, request.0.db_role)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/db/{owner}/{db}/user/list",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 200, description = "ok"),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn list(
    user: UserId,
    State(db_pool): State<DbPool>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<Vec<DbUser>>)> {
    let name = format!("{}/{}", owner, db);
    let database = db_pool.find_user_db(user.0, &name)?;
    let users = db_pool
        .db_users(database.db_id.unwrap())?
        .into_iter()
        .map(|(user, role)| DbUser { user, role })
        .collect();

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/user/{username}/remove",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
        ("username" = String, Path, description = "user name"),
    ),
    responses(
         (status = 204, description = "user removed"),
         (status = 401, description = "unauthorized"),
         (status = 403, description = "must be admin / cannot remove db owner"),
         (status = 404, description = "user / db not found"),
    )
)]
pub(crate) async fn remove(
    user: UserId,
    State(db_pool): State<DbPool>,
    Path((owner, db, username)): Path<(String, String, String)>,
) -> ServerResponse {
    if owner == username {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "cannot remove db owner",
        ));
    }

    let db_name = format!("{}/{}", owner, db);
    let db_id = db_pool.find_db_id(&db_name)?;
    let db_user = db_pool.db_user_id(db_id, &username)?;

    if user.0 != db_user && !db_pool.is_db_admin(user.0, db_id)? {
        return Err(ServerError::new(
            StatusCode::FORBIDDEN,
            "must be a db admin",
        ));
    }

    db_pool.remove_db_user(db_id, db_user)?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn derived_from_clone() {
        let db_role = DbUserRole::Admin;
        let other = db_role.clone();
        let res = db_role == other;

        assert!(res);
    }
}
