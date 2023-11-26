use crate::api::Api;
use crate::db::DbPool;
use crate::db::User;
use crate::error::ServerError;
use crate::logger;
use crate::password::Password;
use axum::body;
use axum::extract::FromRef;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing;
use axum::Json;
use axum::Router;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::broadcast::Sender;
use tower::ServiceBuilder;
use tower_http::map_request_body::MapRequestBodyLayer;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

#[derive(Clone)]
struct ServerState {
    db_pool: DbPool,
    shutdown_sender: Sender<()>,
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct UserCredentials {
    pub(crate) name: String,
    pub(crate) password: String,
}

#[derive(Default, Serialize, ToSchema)]
pub(crate) struct UserToken(String);

impl FromRef<ServerState> for DbPool {
    fn from_ref(input: &ServerState) -> Self {
        input.db_pool.clone()
    }
}

impl FromRef<ServerState> for Sender<()> {
    fn from_ref(input: &ServerState) -> Self {
        input.shutdown_sender.clone()
    }
}

pub(crate) fn app(shutdown_sender: Sender<()>, db_pool: DbPool) -> Router {
    let logger = ServiceBuilder::new()
        .layer(MapRequestBodyLayer::new(body::boxed))
        .layer(middleware::from_fn(logger::logger));

    let state = ServerState {
        db_pool,
        shutdown_sender,
    };

    Router::new()
        .merge(SwaggerUi::new("/openapi").url("/openapi/openapi.json", Api::openapi()))
        .route("/shutdown", routing::get(shutdown))
        .route("/error", routing::get(test_error))
        .route("/create_user", routing::post(create_user))
        .route("/login", routing::post(login))
        .layer(logger)
        .with_state(state)
}

#[utoipa::path(post,
    path = "/create_user",
    request_body = CreateUser,
    responses(
         (status = 201, description = "User created"),
         (status = 461, description = "Name too short (<3)"),
         (status = 462, description = "Password too short (<8)"),
         (status = 463, description = "User already exists")
    )
)]
pub(crate) async fn create_user(
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

    db_pool.create_user(User {
        db_id: None,
        name: request.name.clone(),
        password: pswd.password.to_vec(),
        salt: pswd.user_salt.to_vec(),
        token: String::new(),
    })?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(post,
    path = "/login",
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

async fn test_error() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn shutdown(State(shutdown_sender): State<Sender<()>>) -> StatusCode {
    match shutdown_sender.send(()) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPoolImpl;
    use crate::db::ServerDb;
    use axum::body::Body;
    use axum::http::Request;
    use axum::http::StatusCode;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_db_pool() -> anyhow::Result<DbPool> {
        Ok(DbPool(Arc::new(DbPoolImpl {
            server_db: ServerDb::new("memory:test")?,
            pool: HashMap::new(),
        })))
    }

    #[tokio::test]
    async fn shutdown() -> anyhow::Result<()> {
        let (shutdown_sender, _shutdown_receiver) = tokio::sync::broadcast::channel::<()>(1);
        let db_pool = test_db_pool()?;
        let app = app(shutdown_sender, db_pool);
        let request = Request::builder().uri("/shutdown").body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn bad_shutdown() -> anyhow::Result<()> {
        let db_pool = test_db_pool()?;
        let app = app(Sender::<()>::new(1), db_pool);
        let request = Request::builder().uri("/shutdown").body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }

    #[tokio::test]
    async fn missing_endpoint() -> anyhow::Result<()> {
        let db_pool = test_db_pool()?;
        let app = app(Sender::<()>::new(1), db_pool);
        let request = Request::builder().uri("/missing").body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        Ok(())
    }

    #[test]
    fn generate_openapi_schema() {
        let schema = Api::openapi().to_pretty_json().unwrap();
        let mut file = File::create("openapi/schema.json").unwrap();
        file.write_all(schema.as_bytes()).unwrap();
    }
}
