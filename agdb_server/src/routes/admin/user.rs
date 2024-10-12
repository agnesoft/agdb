use crate::config::Config;
use crate::db_pool::DbPool;
use crate::db_pool::ServerUser;
use crate::error_code::ErrorCode;
use crate::password;
use crate::password::Password;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use agdb_api::UserCredentials;
use agdb_api::UserStatus;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(post,
    path = "/api/v1/admin/user/{username}/add",
    operation_id = "admin_user_add",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "desired user name"),
    ),
    request_body = UserCredentials,
    responses(
         (status = 201, description = "user created"),
         (status = 401, description = "unauthorized"),
         (status = 461, description = "password too short (<8)"),
         (status = 462, description = "name too short (<3)"),
         (status = 463, description = "user exists"),
    )
)]
pub(crate) async fn add(
    _admin_id: AdminId,
    State(db_pool): State<DbPool>,
    Path(username): Path<String>,
    Json(request): Json<UserCredentials>,
) -> ServerResponse {
    password::validate_username(&username)?;
    password::validate_password(&request.password)?;

    if db_pool.find_user_id(&username).await.is_ok() {
        return Err(ErrorCode::UserExists.into());
    }

    let pswd = Password::create(&username, &request.password);

    db_pool
        .add_user(ServerUser {
            db_id: None,
            username: username.clone(),
            password: pswd.password.to_vec(),
            salt: pswd.user_salt.to_vec(),
            token: String::new(),
        })
        .await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(put,
    path = "/api/v1/admin/user/{username}/change_password",
    operation_id = "admin_user_change_password",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "user name"),
    ),
    request_body = UserCredentials,
    responses(
         (status = 201, description = "password changed"),
         (status = 401, description = "unauthorized"),
         (status = 461, description = "password too short (<8)"),
         (status = 464, description = "user not found"),
    )
)]
pub(crate) async fn change_password(
    _admin_id: AdminId,
    State(db_pool): State<DbPool>,
    Path(username): Path<String>,
    Json(request): Json<UserCredentials>,
) -> ServerResponse {
    let user = db_pool.find_user(&username).await?;
    db_pool.change_password(user, &request.password).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/admin/user/list",
    operation_id = "admin_user_list",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 200, description = "ok", body = Vec<UserStatus>),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn list(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
) -> ServerResponse<(StatusCode, Json<Vec<UserStatus>>)> {
    let users = db_pool.find_users().await?;
    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(post,
    path = "/api/v1/admin/user/{username}/logout",
    operation_id = "admin_user_logout",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "user name"),
    ),
    responses(
         (status = 201, description = "user logged out"),
         (status = 401, description = "admin only"),
         (status = 404, description = "user not found"),
    )
)]
pub(crate) async fn logout(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    Path(username): Path<String>,
) -> ServerResponse {
    let mut user = db_pool.find_user(&username).await?;
    user.token = String::new();
    db_pool.save_user(user).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(delete,
    path = "/api/v1/admin/user/{username}/remove",
    operation_id = "admin_user_remove",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "user name"),
    ),
    responses(
         (status = 204, description = "user removed", body = Vec<UserStatus>),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "user not found"),
    )
)]
pub(crate) async fn remove(
    _admin: AdminId,
    State(db_pool): State<DbPool>,
    State(config): State<Config>,
    Path(username): Path<String>,
) -> ServerResponse {
    db_pool.remove_user(&username, &config).await?;

    Ok(StatusCode::NO_CONTENT)
}
