use crate::action::change_password::ChangePassword as ChangePasswordAction;
use crate::cluster::Cluster;
use crate::config::Config;
use crate::password;
use crate::password::Password;
use crate::routes::ServerResult;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::user_id::UserId;
use crate::user_id::UserName;
use agdb::DbId;
use agdb_api::ChangePassword;
use agdb_api::UserLogin;
use agdb_api::UserStatus;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

pub(crate) async fn do_login(
    server_db: &ServerDb,
    username: &str,
    password: &str,
) -> ServerResult<(String, Option<DbId>)> {
    let user = server_db
        .user(username)
        .await
        .map_err(|_| ServerError::new(StatusCode::UNAUTHORIZED, "unuauthorized"))?;
    let pswd = Password::new(&user.username, &user.password, &user.salt)?;

    if !pswd.verify_password(password) {
        return Err(ServerError::new(StatusCode::UNAUTHORIZED, "unuauthorized"));
    }

    let user_id = user.db_id;
    let mut token = server_db.user_token(user_id.unwrap_or_default()).await?;

    if token.is_empty() {
        let token_uuid = Uuid::new_v4();
        token = token_uuid.to_string();
    }

    Ok((token, user_id))
}

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
    State(server_db): State<ServerDb>,
    Json(request): Json<UserLogin>,
) -> ServerResponse<(StatusCode, Json<String>)> {
    let (token, user_id) = do_login(&server_db, &request.username, &request.password).await?;

    if let Some(user_id) = user_id {
        server_db.save_token(user_id, &token).await?;
    }

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
pub(crate) async fn logout(user: UserId, State(server_db): State<ServerDb>) -> ServerResponse {
    server_db.save_token(user.0, "").await?;

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
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Json(request): Json<ChangePassword>,
) -> ServerResponse {
    let user = server_db.user_by_id(user.0).await?;
    let old_pswd = Password::new(&user.username, &user.password, &user.salt)?;

    if !old_pswd.verify_password(&request.password) {
        return Err(ServerError::new(StatusCode::UNAUTHORIZED, "unuauthorized"));
    }

    password::validate_password(&request.new_password)?;
    let pswd = Password::create(&user.username, &request.new_password);

    cluster
        .append(ChangePasswordAction {
            user: user.username,
            new_password: pswd.password.to_vec(),
            new_salt: pswd.user_salt.to_vec(),
        })
        .await?;

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
