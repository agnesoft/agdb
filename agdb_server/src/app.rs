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

    let api_v1 = Router::new()
        .route("/test_error", routing::get(routes::test_error))
        .route("/status", routing::get(routes::status))
        .route("/admin/shutdown", routing::post(routes::admin::shutdown))
        .route("/admin/user/list", routing::get(routes::admin::user::list))
        .route(
            "/admin/user/:user/add",
            routing::put(routes::admin::user::add),
        )
        .route(
            "/admin/user/:user/change_password",
            routing::put(routes::admin::user::change_password),
        )
        .route("/admin/db/list", routing::get(routes::admin::db::list))
        .route(
            "/admin/db/:user/:db/add",
            routing::post(routes::admin::db::add),
        )
        .route(
            "/admin/db/:user/:db/delete",
            routing::post(routes::admin::db::delete),
        )
        .route(
            "/admin/db/:user/:db/exec",
            routing::post(routes::admin::db::exec),
        )
        .route(
            "/admin/db/:user/:db/remove",
            routing::post(routes::admin::db::remove),
        )
        .route(
            "/admin/db/:user/:db/user/list",
            routing::post(routes::admin::db::user::list),
        )
        .route(
            "/admin/db/:user/:db/user/:other/add",
            routing::post(routes::admin::db::user::add),
        )
        .route(
            "/admin/db/:user/:db/user/:other/remove",
            routing::post(routes::admin::db::user::remove),
        )
        .route("/db/list", routing::get(routes::db::list))
        .route("/db/:user/:db/add", routing::post(routes::db::add))
        .route("/db/:user/:db/delete", routing::post(routes::db::delete))
        .route("/db/:user/:db/exec", routing::post(routes::db::exec))
        .route(
            "/db/:user/:db/optimize",
            routing::post(routes::db::optimize),
        )
        .route("/db/:user/:db/remove", routing::post(routes::db::remove))
        .route("/db/:user/:db/rename", routing::post(routes::db::rename))
        .route(
            "/db/:user/:db/user/list",
            routing::post(routes::db::user::list),
        )
        .route(
            "/db/:user/:db/user/:other/add",
            routing::post(routes::db::user::add),
        )
        .route(
            "/db/:user/:db/user/:other/remove",
            routing::post(routes::db::user::remove),
        )
        .route("/user/:user/login", routing::post(routes::user::login))
        .route(
            "/user/:user/change_password",
            routing::put(routes::user::change_password),
        );

    Router::new()
        .merge(RapiDoc::with_openapi("/api/v1/openapi.json", Api::openapi()).path("/api/v1"))
        .nest("/api/v1", api_v1)
        .layer(middleware::from_fn(logger::logger))
        .with_state(state)
}
