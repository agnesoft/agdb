use crate::db::DbPool;
use crate::db::DbUser;
use crate::password::Password;
use crate::routes::user::UserCredentials;
use crate::server_error::ServerError;
use crate::user_id::AdminId;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use tokio::sync::broadcast::Sender;

#[utoipa::path(post,
    path = "/api/v1/admin/create_user",
    request_body = UserCredentials,
    responses(
         (status = 201, description = "User created"),
         (status = 461, description = "Name too short (<3)"),
         (status = 462, description = "Password too short (<8)"),
         (status = 463, description = "User already exists")
    )
)]
pub(crate) async fn create_user(
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

#[utoipa::path(get,
    path = "/api/v1/admin/shutdown",
    security(("Token" = [])),
    responses(
         (status = 200, description = "Server is shutting down"),
    )
)]
pub(crate) async fn shutdown(
    _admin_id: AdminId,
    State(shutdown_sender): State<Sender<()>>,
) -> StatusCode {
    match shutdown_sender.send(()) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::DbId;

    #[tokio::test]
    async fn shutdown_test() -> anyhow::Result<()> {
        let (shutdown_sender, _shutdown_receiver) = tokio::sync::broadcast::channel::<()>(1);

        let status = shutdown(AdminId(DbId(0)), State(shutdown_sender)).await;

        assert_eq!(status, StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn bad_shutdown() -> anyhow::Result<()> {
        let shutdown_sender = Sender::<()>::new(1);

        let status = shutdown(AdminId(DbId(0)), State(shutdown_sender)).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }
}
