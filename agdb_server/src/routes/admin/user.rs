use crate::db::DbPool;
use crate::db::ServerUser;
use crate::error_code::ErrorCode;
use crate::password;
use crate::password::Password;
use crate::routes::user::UserCredentials;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
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
    path = "/api/v1/admin/user/change_password",
    security(("Token" = [])),
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
    Json(request): Json<UserCredentials>,
) -> ServerResponse {
    password::validate_password(&request.password)?;

    let mut user = db_pool.find_user(&request.name)?;
    let pswd = Password::create(&request.name, &request.password);
    user.password = pswd.password.to_vec();
    user.salt = pswd.user_salt.to_vec();

    db_pool.save_user(user)?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/admin/user/create",
    request_body = UserCredentials,
    security(("Token" = [])),
    responses(
         (status = 201, description = "user created"),
         (status = 401, description = "unauthorized"),
         (status = 461, description = "password too short (<8)"),
         (status = 462, description = "name too short (<3)"),
         (status = 463, description = "user already exists")
    )
)]
pub(crate) async fn create(
    _admin_id: AdminId,
    State(db_pool): State<DbPool>,
    Json(request): Json<UserCredentials>,
) -> ServerResponse {
    password::validate_username(&request.name)?;
    password::validate_password(&request.password)?;

    if db_pool.find_user_id(&request.name).is_ok() {
        return Err(ErrorCode::UserExists.into());
    }

    let pswd = Password::create(&request.name, &request.password);

    db_pool.create_user(ServerUser {
        db_id: None,
        name: request.name.clone(),
        password: pswd.password.to_vec(),
        salt: pswd.user_salt.to_vec(),
        token: String::new(),
    })?;

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
