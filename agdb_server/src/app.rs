use crate::api::Api;
use crate::db::DbPool;
use crate::logger;
use crate::routes;
use crate::server_state::ServerState;
use axum::middleware;
use axum::routing;
use axum::Router;
use tokio::sync::broadcast::Sender;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub(crate) fn app(shutdown_sender: Sender<()>, db_pool: DbPool) -> Router {
    let state = ServerState {
        db_pool,
        shutdown_sender,
    };

    let admin_router_v1 = Router::new().route("/shutdown", routing::get(routes::admin::shutdown));

    let user_router_v1 = Router::new()
        .route("/create", routing::post(routes::user::create))
        .route(
            "/change_password",
            routing::post(routes::user::change_password),
        )
        .route("/login", routing::post(routes::user::login));

    let db_router_v1 = Router::new()
        .route("/add", routing::post(routes::db::add))
        .route("/delete", routing::post(routes::db::delete))
        .route("/list", routing::get(routes::db::list))
        .route("/remove", routing::post(routes::db::remove));

    Router::new()
        .nest(
            "/api",
            Router::new().nest(
                "/v1",
                Router::new()
                    .merge(SwaggerUi::new("/openapi").url("/openapi/openapi.json", Api::openapi()))
                    .route("/test_error", routing::get(routes::test_error))
                    .nest("/admin", admin_router_v1)
                    .nest("/user", user_router_v1)
                    .nest("/db", db_router_v1),
            ),
        )
        .layer(middleware::from_fn(logger::logger))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Api;
    use crate::db::DbPoolImpl;
    use crate::db::ServerDb;
    use crate::server_error::ServerResult;
    use axum::body::Body;
    use axum::http::Request;
    use axum::http::StatusCode;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;
    use std::sync::RwLock;
    use tower::ServiceExt;
    use utoipa::OpenApi;

    fn test_db_pool() -> ServerResult<DbPool> {
        Ok(DbPool(Arc::new(DbPoolImpl {
            server_db: ServerDb::new("memory:test")?,
            pool: RwLock::new(HashMap::new()),
        })))
    }

    #[tokio::test]
    async fn missing_endpoint() -> ServerResult {
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
