use crate::api::Api;
use crate::config::Config;
use crate::db_pool::DbPool;
use crate::logger;
use crate::routes;
use crate::server_state::ServerState;
use axum::middleware;
use axum::routing;
use axum::Router;
use tokio::sync::broadcast::Sender;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

pub(crate) fn app(config: Config, shutdown_sender: Sender<()>, db_pool: DbPool) -> Router {
    let state = ServerState {
        db_pool,
        config,
        shutdown_sender,
    };

    let admin_db_user_router_v1 = Router::new()
        .route("/add", routing::post(routes::admin::db::user::add))
        .route("/list", routing::get(routes::admin::db::user::list))
        .route("/remove", routing::post(routes::admin::db::user::remove));

    let admin_db_router_v1 = Router::new()
        .route("/add", routing::post(routes::admin::db::add))
        .route("/delete", routing::post(routes::admin::db::delete))
        .route("/list", routing::get(routes::admin::db::list))
        .route("/remove", routing::post(routes::admin::db::remove))
        .nest("/user", admin_db_user_router_v1);

    let admin_user_router_v1 = Router::new()
        .route(
            "/change_password",
            routing::post(routes::admin::user::change_password),
        )
        .route("/create", routing::post(routes::admin::user::create))
        .route("/list", routing::get(routes::admin::user::list));

    let admin_router_v1 = Router::new()
        .route("/shutdown", routing::get(routes::admin::shutdown))
        .nest("/db", admin_db_router_v1)
        .nest("/user", admin_user_router_v1);

    let user_router_v1 = Router::new()
        .route(
            "/change_password",
            routing::post(routes::user::change_password),
        )
        .route("/login", routing::post(routes::user::login));

    let db_user_router_v1 = Router::new()
        .route("/add", routing::post(routes::db::user::add))
        .route("/list", routing::get(routes::db::user::list))
        .route("/remove", routing::post(routes::db::user::remove));

    let db_router_v1 = Router::new()
        .route("/add", routing::post(routes::db::add))
        .route("/delete", routing::post(routes::db::delete))
        .route("/list", routing::get(routes::db::list))
        .route("/remove", routing::post(routes::db::remove))
        .nest("/user", db_user_router_v1);

    Router::new()
        .merge(RapiDoc::with_openapi("/api/v1/openapi.json", Api::openapi()).path("/api/v1"))
        .nest(
            "/api",
            Router::new().nest(
                "/v1",
                Router::new()
                    .route("/test_error", routing::get(routes::test_error))
                    .route("/status", routing::get(routes::status))
                    .nest("/admin", admin_router_v1)
                    .nest("/user", user_router_v1)
                    .nest("/db", db_router_v1),
            ),
        )
        .layer(middleware::from_fn(logger::logger))
        .with_state(state)
}
