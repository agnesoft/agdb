use crate::db::DbPool;
use crate::db::DbUser;
use crate::password::Password;
use crate::routes::user::UserCredentials;
use crate::server_error::ServerError;
use crate::user_id::AdminId;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(post,
    path = "/api/v1/admin/user/change_password",
    security(("Token" = [])),
    request_body = UserCredentials,
    responses(
         (status = 200, description = "Password changed"),
         (status = 401, description = "Invalid password"),
         (status = 403, description = "User not found"),
         (status = 462, description = "Password too short (<8)"),
    )
)]
pub(crate) async fn change_password(
    _admin_id: AdminId,
    State(db_pool): State<DbPool>,
    Json(request): Json<UserCredentials>,
) -> Result<StatusCode, ServerError> {
    if request.password.len() < 8 {
        return Ok(StatusCode::from_u16(462_u16)?);
    }

    let mut user = db_pool
        .find_user(&request.name)
        .map_err(|_| ServerError::new(StatusCode::FORBIDDEN, "User not found"))?;

    let pswd = Password::create(&request.name, &request.password);
    user.password = pswd.password.to_vec();
    user.salt = pswd.user_salt.to_vec();

    db_pool.save_user(user)?;

    Ok(StatusCode::OK)
}

#[utoipa::path(post,
    path = "/api/v1/admin/user/create",
    request_body = UserCredentials,
    responses(
         (status = 201, description = "User created"),
         (status = 461, description = "Name too short (<3)"),
         (status = 462, description = "Password too short (<8)"),
         (status = 463, description = "User already exists")
    )
)]
pub(crate) async fn create(
    _admin_id: AdminId,
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
