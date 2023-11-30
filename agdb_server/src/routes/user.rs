use crate::db::DbPool;
use crate::db::DbUser;
use crate::password::Password;
use crate::server_error::ServerError;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub(crate) struct UserCredentials {
    pub(crate) name: String,
    pub(crate) password: String,
}

#[derive(Default, Serialize, ToSchema)]
pub(crate) struct UserToken(String);

#[derive(Deserialize, ToSchema)]
pub(crate) struct ChangePassword {
    pub(crate) name: String,
    pub(crate) password: String,
    pub(crate) new_password: String,
}

#[utoipa::path(post,
    path = "api/v1/user/login",
    request_body = UserCredentials,
    responses(
         (status = 200, description = "Login successful", body = UserToken),
         (status = 401, description = "Bad password"),
         (status = 403, description = "User not found")
    )
)]
pub(crate) async fn login(
    State(db_pool): State<DbPool>,
    Json(request): Json<UserCredentials>,
) -> Result<(StatusCode, Json<UserToken>), ServerError> {
    let user = db_pool.find_user(&request.name);

    if user.is_err() {
        return Ok((StatusCode::FORBIDDEN, Json::default()));
    }

    let user = user?;
    let pswd = Password::new(&user.name, &user.password, &user.salt)?;

    if !pswd.verify_password(&request.password) {
        return Ok((StatusCode::UNAUTHORIZED, Json::default()));
    }

    let token_uuid = Uuid::new_v4();
    let token = token_uuid.to_string();
    db_pool.save_token(user.db_id.unwrap(), &token)?;

    Ok((StatusCode::OK, Json(UserToken(token))))
}

#[utoipa::path(post,
    path = "api/v1/user/create",
    request_body = UserCredentials,
    responses(
         (status = 201, description = "User created"),
         (status = 461, description = "Name too short (<3)"),
         (status = 462, description = "Password too short (<8)"),
         (status = 463, description = "User already exists")
    )
)]
pub(crate) async fn create(
    State(db_pool): State<DbPool>,
    Json(request): Json<UserCredentials>,
) -> Result<StatusCode, ServerError> {
    if request.name.len() < 3 {
        return Ok(StatusCode::from_u16(461_u16)?);
    }

    if request.password.len() < 8 {
        return Ok(StatusCode::from_u16(462_u16)?);
    }

    if db_pool.find_user(&request.name).is_ok() {
        return Ok(StatusCode::from_u16(463_u16)?);
    }

    let pswd = Password::create(&request.name, &request.password);

    db_pool.create_user(DbUser {
        db_id: None,
        name: request.name.clone(),
        password: pswd.password.to_vec(),
        salt: pswd.user_salt.to_vec(),
        token: String::new(),
    })?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "api/v1/user/change_password",
    request_body = ChangePassword,
    responses(
         (status = 200, description = "Password changed"),
         (status = 401, description = "Invalid password"),
         (status = 403, description = "User not found"),
         (status = 462, description = "Password too short (<8)"),
    )
)]
pub(crate) async fn change_password(
    State(db_pool): State<DbPool>,
    Json(request): Json<ChangePassword>,
) -> Result<StatusCode, ServerError> {
    if request.new_password.len() < 8 {
        return Ok(StatusCode::from_u16(462_u16)?);
    }

    let mut user = db_pool
        .find_user(&request.name)
        .map_err(|_| ServerError::new(StatusCode::FORBIDDEN, "User not found"))?;

    let old_pswd = Password::new(&user.name, &user.password, &user.salt)?;

    if !old_pswd.verify_password(&request.password) {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    let pswd = Password::create(&request.name, &request.new_password);
    user.password = pswd.password.to_vec();
    user.salt = pswd.user_salt.to_vec();

    db_pool.save_user(user)?;

    Ok(StatusCode::OK)
}
