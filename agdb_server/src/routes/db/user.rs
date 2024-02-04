use crate::db_pool::DbPool;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
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
pub(crate) struct DbUserRoleParam {
    pub(crate) db_role: DbUserRole,
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/user/{username}/add",
    operation_id = "db_user_add",
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
    db_pool
        .add_db_user(&owner, &db, &username, request.0.db_role, user.0)
        .await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/db/{owner}/{db}/user/list",
    operation_id = "db_user_list",
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
    let users = db_pool.db_users(&owner, &db, user.0).await?;

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/user/{username}/remove",
    operation_id = "db_user_remove",
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
    db_pool
        .remove_db_user(&owner, &db, &username, user.0)
        .await?;

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
