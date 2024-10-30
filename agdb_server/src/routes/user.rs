use crate::config::Config;
use crate::db_pool::DbPool;
use crate::password::Password;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use crate::user_id::UserName;
use agdb_api::ChangePassword;
use agdb_api::UserLogin;
use agdb_api::UserStatus;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(post,
    path = "/api/v1/user/login",
    operation_id = "user_login",
    tag = "agdb",
    request_body = UserLogin,
    responses(
         (status = 200, description = "login successful", body = String),
         (status = 401, description = "invalid credentials"),
    )
)]
pub(crate) async fn login(
    State(db_pool): State<DbPool>,
    Json(request): Json<UserLogin>,
) -> ServerResponse<(StatusCode, Json<String>)> {
    let user = db_pool
        .find_user(&request.username)
        .await
        .map_err(|_| ServerError::new(StatusCode::UNAUTHORIZED, "unuauthorized"))?;
    let pswd = Password::new(&user.username, &user.password, &user.salt)?;

    if !pswd.verify_password(&request.password) {
        return Err(ServerError::new(StatusCode::UNAUTHORIZED, "unuauthorized"));
    }

    let token = db_pool.user_token(user.db_id.unwrap()).await?;

    Ok((StatusCode::OK, Json(token)))
}

#[utoipa::path(post,
    path = "/api/v1/user/logout",
    operation_id = "user_logout",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 201, description = "user logged out"),
         (status = 401, description = "invalid credentials")
    )
)]
pub(crate) async fn logout(user: UserId, State(db_pool): State<DbPool>) -> ServerResponse {
    let mut user = db_pool.get_user(user.0).await?;
    user.token = String::new();
    db_pool.save_user(user).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(put,
    path = "/api/v1/user/change_password",
    operation_id = "user_change_password",
    tag = "agdb",
    security(("Token" = [])),
    request_body = ChangePassword,
    responses(
         (status = 201, description = "password changed"),
         (status = 401, description = "invalid credentials"),
         (status = 461, description = "password too short (<8)"),
    )
)]
pub(crate) async fn change_password(
    user: UserId,
    State(db_pool): State<DbPool>,
    Json(request): Json<ChangePassword>,
) -> ServerResponse {
    let user = db_pool.get_user(user.0).await?;
    let old_pswd = Password::new(&user.username, &user.password, &user.salt)?;

    if !old_pswd.verify_password(&request.password) {
        return Err(ServerError::new(StatusCode::UNAUTHORIZED, "unuauthorized"));
    }

    db_pool.change_password(user, &request.new_password).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(get,
    path = "/api/v1/user/status",
    operation_id = "user_status",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 200, description = "User status", body = UserStatus),
         (status = 401, description = "unauthorized"),
    )
)]
pub(crate) async fn status(
    _user: UserId,
    username: UserName,
    State(config): State<Config>,
) -> ServerResponse<(StatusCode, Json<UserStatus>)> {
    Ok((
        StatusCode::OK,
        Json(UserStatus {
            admin: username.0 == config.admin,
            name: username.0,
            login: true,
        }),
    ))
}
