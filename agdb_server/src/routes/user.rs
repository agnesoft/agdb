use crate::db_pool::DbPool;
use crate::password::Password;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub(crate) struct UserCredentials {
    pub(crate) password: String,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct UserLogin {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct ChangePassword {
    pub(crate) password: String,
    pub(crate) new_password: String,
}

#[utoipa::path(post,
    path = "/api/v1/user/login",
    operation_id = "user_login",
    request_body = UserLogin,
    responses(
         (status = 200, description = "login successful", body = String),
         (status = 401, description = "invalid credentials"),
    )
)]
pub(crate) async fn login(
    State(db_pool): State<DbPool>,
    Json(request): Json<UserLogin>,
) -> ServerResponse<(StatusCode, String)> {
    let user = db_pool
        .find_user(&request.username)
        .map_err(|_| ServerError::new(StatusCode::UNAUTHORIZED, "unuauthorized"))?;
    let pswd = Password::new(&user.name, &user.password, &user.salt)?;

    if !pswd.verify_password(&request.password) {
        return Err(ServerError::new(StatusCode::UNAUTHORIZED, "unuauthorized"));
    }

    let token_uuid = Uuid::new_v4();
    let token = token_uuid.to_string();
    db_pool.save_token(user.db_id.unwrap(), &token)?;

    Ok((StatusCode::OK, token))
}

#[utoipa::path(put,
    path = "/api/v1/user/{username}/change_password",
    operation_id = "user_change_password",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "username"),
    ),
    request_body = ChangePassword,
    responses(
         (status = 201, description = "password changed"),
         (status = 401, description = "invalid credentials"),
         (status = 404, description = "user not found"),
         (status = 461, description = "password too short (<8)"),
    )
)]
pub(crate) async fn change_password(
    State(db_pool): State<DbPool>,
    Path(username): Path<String>,
    Json(request): Json<ChangePassword>,
) -> ServerResponse {
    let user = db_pool.find_user(&username)?;
    let old_pswd = Password::new(&user.name, &user.password, &user.salt)?;

    if !old_pswd.verify_password(&request.password) {
        return Err(ServerError::new(StatusCode::UNAUTHORIZED, "unuauthorized"));
    }

    db_pool.change_password(user, &request.new_password)?;

    Ok(StatusCode::CREATED)
}
