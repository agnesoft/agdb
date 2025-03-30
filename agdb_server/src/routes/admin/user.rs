use crate::action::change_password::ChangePassword as ChangePasswordAction;
use crate::action::user_add::UserAdd;
use crate::action::user_delete::UserDelete;
use crate::cluster::Cluster;
use crate::error_code::ErrorCode;
use crate::password;
use crate::password::Password;
use crate::server_db::ServerDb;
use crate::server_error::ServerResponse;
use crate::user_id::AdminId;
use agdb_api::UserCredentials;
use agdb_api::UserStatus;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

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
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path(username): Path<String>,
    Json(request): Json<UserCredentials>,
) -> ServerResponse<impl IntoResponse> {
    password::validate_username(&username)?;
    password::validate_password(&request.password)?;

    if server_db.find_user_id(&username).await?.is_some() {
        return Err(ErrorCode::UserExists.into());
    }

    let pswd = Password::create(&username, &request.password);

    let (commit_index, _result) = cluster
        .exec(UserAdd {
            user: username,
            password: pswd.password.to_vec(),
            salt: pswd.user_salt.to_vec(),
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
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
         (status = 464, description = "user not found"),
    )
)]
pub(crate) async fn change_password(
    _admin_id: AdminId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path(username): Path<String>,
    Json(request): Json<UserCredentials>,
) -> ServerResponse<impl IntoResponse> {
    let _user = server_db.user_id(&username).await?;
    let pswd = Password::create(&username, &request.password);

    let (commit_index, _result) = cluster
        .exec(ChangePasswordAction {
            user: username.to_string(),
            new_password: pswd.password.to_vec(),
            new_salt: pswd.user_salt.to_vec(),
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
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
    State(server_db): State<ServerDb>,
) -> ServerResponse<(StatusCode, Json<Vec<UserStatus>>)> {
    Ok((StatusCode::OK, Json(server_db.user_statuses().await?)))
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
    State(server_db): State<ServerDb>,
    Path(username): Path<String>,
) -> ServerResponse {
    let user_id = server_db.user_id(&username).await?;
    server_db.save_token(user_id, "").await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/api/v1/admin/user/logout_all",
    operation_id = "admin_user_logout_all",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 201, description = "users logged out"),
         (status = 401, description = "admin only"),
    )
)]
pub(crate) async fn logout_all(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
) -> ServerResponse {
    server_db.reset_tokens().await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(delete,
    path = "/api/v1/admin/user/{username}/delete",
    operation_id = "admin_user_delete",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "user name"),
    ),
    responses(
         (status = 204, description = "user deleted", body = Vec<UserStatus>),
         (status = 401, description = "unauthorized"),
         (status = 404, description = "user not found"),
    )
)]
pub(crate) async fn delete(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path(username): Path<String>,
) -> ServerResponse<impl IntoResponse> {
    server_db.user_id(&username).await?;

    let (commit_index, _result) = cluster
        .exec(UserDelete {
            user: username.to_string(),
        })
        .await?;

    Ok((
        StatusCode::NO_CONTENT,
        [("commit-index", commit_index.to_string())],
    ))
}
