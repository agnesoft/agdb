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
use utoipa_rapidoc::RapiDoc;

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
        .merge(RapiDoc::with_openapi("/api/v1/openapi.json", Api::openapi()).path("/api/v1"))
        .nest(
            "/api",
            Router::new().nest(
                "/v1",
                Router::new()
                    .route("/test_error", routing::get(routes::test_error))
                    .nest("/admin", admin_router_v1)
                    .nest("/user", user_router_v1)
                    .nest("/db", db_router_v1),
            ),
        )
        .layer(middleware::from_fn(logger::logger))
        .with_state(state)
}
