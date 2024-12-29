use crate::action::db_user_add::DbUserAdd;
use crate::action::db_user_remove::DbUserRemove;
use crate::cluster::Cluster;
use crate::server_db::ServerDb;
use crate::server_error::permission_denied;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub(crate) struct DbUserRoleParam {
    pub(crate) db_role: DbUserRole,
}

#[utoipa::path(put,
    path = "/api/v1/db/{owner}/{db}/user/{username}/add",
    operation_id = "db_user_add",
    tag = "agdb",
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
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path((owner, db, username)): Path<(String, String, String)>,
    request: Query<DbUserRoleParam>,
) -> ServerResponse<impl IntoResponse> {
    if owner == username {
        return Err(permission_denied("cannot change role of db owner"));
    }

    let db_id = server_db.user_db_id(user.0, &owner, &db).await?;

    if !server_db.is_db_admin(user.0, db_id).await? {
        return Err(permission_denied("admin only"));
    }

    let _ = server_db.user_id(&username).await?;

    let (commit_index, _result) = cluster
        .exec(DbUserAdd {
            owner,
            db,
            user: username,
            db_role: request.db_role,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(get,
    path = "/api/v1/db/{owner}/{db}/user/list",
    operation_id = "db_user_list",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("owner" = String, Path, description = "db owner user name"),
        ("db" = String, Path, description = "db name"),
    ),
    responses(
         (status = 200, description = "ok", body = Vec<DbUser>),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "db not found"),
    )
)]
pub(crate) async fn list(
    user: UserId,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResponse<(StatusCode, Json<Vec<DbUser>>)> {
    let db_id = server_db.user_db_id(user.0, &owner, &db).await?;

    Ok((StatusCode::OK, Json(server_db.db_users(db_id).await?)))
}

#[utoipa::path(post,
    path = "/api/v1/db/{owner}/{db}/user/{username}/remove",
    operation_id = "db_user_remove",
    tag = "agdb",
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
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path((owner, db, username)): Path<(String, String, String)>,
) -> ServerResponse<impl IntoResponse> {
    if owner == username {
        return Err(permission_denied("cannot remove owner"));
    }

    let db_id = server_db.user_db_id(user.0, &owner, &db).await?;
    let user_id = server_db.user_id(&username).await?;

    if user.0 != user_id && !server_db.is_db_admin(user.0, db_id).await? {
        return Err(permission_denied("admin only"));
    }

    let (commit_index, _result) = cluster
        .exec(DbUserRemove {
            owner,
            db,
            user: username,
        })
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("commit-index", commit_index.to_string())],
    ))
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
