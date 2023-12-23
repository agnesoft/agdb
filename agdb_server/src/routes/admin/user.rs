use crate::db_pool::DbPool;
use crate::db_pool::ServerUser;
use crate::error_code::ErrorCode;
use crate::password;
use crate::password::Password;
use crate::routes::user::UserCredentials;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub(crate) struct UserStatus {
    pub(crate) name: String,
}

#[utoipa::path(post,
    path = "/api/v1/admin/user/{username}/add",
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

    if db_pool.find_user_id(&username).is_ok() {
        return Err(ErrorCode::UserExists.into());
    }

    let pswd = Password::create(&username, &request.password);

    db_pool.add_user(ServerUser {
        db_id: None,
        name: username.clone(),
        password: pswd.password.to_vec(),
        salt: pswd.user_salt.to_vec(),
        token: String::new(),
    })?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(put,
    path = "/api/v1/admin/user/{username}/change_password",
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
    let user = db_pool.find_user(&username)?;
    db_pool.change_password(user, &request.password)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/admin/user/list",
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
    let users = db_pool
        .find_users()?
        .into_iter()
        .map(|name| UserStatus { name })
        .collect();
    Ok((StatusCode::OK, Json(users)))
}
