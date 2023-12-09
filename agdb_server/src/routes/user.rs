use crate::db::DbPool;
use crate::password;
use crate::password::Password;
use crate::server_error::ServerResponse;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub(crate) struct UserCredentials {
    pub(crate) name: String,
    pub(crate) password: String,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct ChangePassword {
    pub(crate) name: String,
    pub(crate) password: String,
    pub(crate) new_password: String,
}

#[utoipa::path(post,
    path = "/api/v1/user/login",
    request_body = UserCredentials,
    responses(
         (status = 200, description = "login successful", body = String),
         (status = 401, description = "invalid credentials"),
         (status = 464, description = "user not found")
    )
)]
pub(crate) async fn login(
    State(db_pool): State<DbPool>,
    Json(request): Json<UserCredentials>,
) -> ServerResponse<(StatusCode, String)> {
    let user = db_pool.find_user(&request.name)?;
    let pswd = Password::new(&user.name, &user.password, &user.salt)?;

    if !pswd.verify_password(&request.password) {
        return Ok((StatusCode::UNAUTHORIZED, String::new()));
    }

    let token_uuid = Uuid::new_v4();
    let token = token_uuid.to_string();
    db_pool.save_token(user.db_id.unwrap(), &token)?;

    Ok((StatusCode::OK, token))
}

#[utoipa::path(post,
    path = "/api/v1/user/change_password",
    security(("Token" = [])),
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
    Json(request): Json<ChangePassword>,
) -> ServerResponse {
    password::validate_password(&request.new_password)?;

    let mut user = db_pool.find_user(&request.name)?;
    let old_pswd = Password::new(&user.name, &user.password, &user.salt)?;

    if !old_pswd.verify_password(&request.password) {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    let new_pswd = Password::create(&request.name, &request.new_password);
    user.password = new_pswd.password.to_vec();
    user.salt = new_pswd.user_salt.to_vec();

    db_pool.save_user(user)?;

    Ok(StatusCode::CREATED)
}
