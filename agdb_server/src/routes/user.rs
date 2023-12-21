use crate::db_pool::DbPool;
use crate::password;
use crate::password::Password;
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
pub(crate) struct ChangePassword {
    pub(crate) password: String,
    pub(crate) new_password: String,
}

#[utoipa::path(post,
    path = "/api/v1/user/{username}/login",
    params(
        ("username" = String, Path, description = "username"),
    ),
    request_body = UserCredentials,
    responses(
         (status = 200, description = "login successful", body = String),
         (status = 401, description = "invalid credentials"),
         (status = 464, description = "user not found")
    )
)]
pub(crate) async fn login(
    State(db_pool): State<DbPool>,
    Path(username): Path<String>,
    Json(request): Json<UserCredentials>,
) -> ServerResponse<(StatusCode, String)> {
    let user = db_pool.find_user(&username)?;
    let pswd = Password::new(&user.name, &user.password, &user.salt)?;

    if !pswd.verify_password(&request.password) {
        return Ok((StatusCode::UNAUTHORIZED, String::new()));
    }

    let token_uuid = Uuid::new_v4();
    let token = token_uuid.to_string();
    db_pool.save_token(user.db_id.unwrap(), &token)?;

    Ok((StatusCode::OK, token))
}

#[utoipa::path(put,
    path = "/api/v1/user/{username}/change_password",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "username"),
    ),
    request_body = ChangePassword,
    responses(
         (status = 201, description = "password changed"),
         (status = 401, description = "invalid credentials"),
         (status = 461, description = "password too short (<8)"),
         (status = 464, description = "user not found"),
    )
)]
pub(crate) async fn change_password(
    State(db_pool): State<DbPool>,
    Path(username): Path<String>,
    Json(request): Json<ChangePassword>,
) -> ServerResponse {
    password::validate_password(&request.new_password)?;

    let mut user = db_pool.find_user(&username)?;
    let old_pswd = Password::new(&user.name, &user.password, &user.salt)?;

    if !old_pswd.verify_password(&request.password) {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    let new_pswd = Password::create(&user.name, &request.new_password);
    user.password = new_pswd.password.to_vec();
    user.salt = new_pswd.user_salt.to_vec();

    db_pool.save_user(user)?;

    Ok(StatusCode::CREATED)
}
