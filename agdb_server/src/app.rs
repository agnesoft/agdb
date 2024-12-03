use crate::api::Api;
use crate::cluster::Cluster;
use crate::config::Config;
use crate::db_pool::DbPool;
use crate::logger;
use crate::routes;
use crate::server_db::ServerDb;
use crate::server_state::ServerState;
use axum::middleware;
use axum::routing;
use axum::Router;
use reqwest::Method;
use tokio::sync::broadcast::Sender;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

pub(crate) fn app(
    cluster: Cluster,
    config: Config,
    db_pool: DbPool,
    server_db: ServerDb,
    shutdown_sender: Sender<()>,
) -> Router {
    let basepath = config.basepath.clone();

    let state = ServerState {
        cluster,
        config,
        db_pool,
        server_db,
        shutdown_sender,
    };

    let api_v1 = Router::new()
        .route("/test_error", routing::get(routes::test_error))
        .route("/status", routing::get(routes::status))
        .route("/admin/shutdown", routing::post(routes::admin::shutdown))
        .route("/admin/status", routing::get(routes::admin::status))
        .route("/admin/user/list", routing::get(routes::admin::user::list))
        .route(
            "/admin/user/:user/logout",
            routing::post(routes::admin::user::logout),
        )
        .route(
            "/admin/user/:user/add",
            routing::post(routes::admin::user::add),
        )
        .route(
            "/admin/user/:user/change_password",
            routing::put(routes::admin::user::change_password),
        )
        .route(
            "/admin/user/:user/remove",
            routing::delete(routes::admin::user::remove),
        )
        .route("/admin/db/list", routing::get(routes::admin::db::list))
        .route(
            "/admin/db/:user/:db/add",
            routing::post(routes::admin::db::add),
        )
        .route(
            "/admin/db/:user/:db/audit",
            routing::get(routes::admin::db::audit),
        )
        .route(
            "/admin/db/:user/:db/backup",
            routing::post(routes::admin::db::backup),
        )
        .route(
            "/admin/db/:user/:db/clear",
            routing::post(routes::admin::db::clear),
        )
        .route(
            "/admin/db/:user/:db/convert",
            routing::post(routes::admin::db::convert),
        )
        .route(
            "/admin/db/:user/:db/copy",
            routing::post(routes::admin::db::copy),
        )
        .route(
            "/admin/db/:user/:db/delete",
            routing::delete(routes::admin::db::delete),
        )
        .route(
            "/admin/db/:user/:db/exec",
            routing::post(routes::admin::db::exec),
        )
        .route(
            "/admin/db/:user/:db/optimize",
            routing::post(routes::admin::db::optimize),
        )
        .route(
            "/admin/db/:user/:db/remove",
            routing::delete(routes::admin::db::remove),
        )
        .route(
            "/admin/db/:user/:db/rename",
            routing::post(routes::admin::db::rename),
        )
        .route(
            "/admin/db/:user/:db/restore",
            routing::post(routes::admin::db::restore),
        )
        .route(
            "/admin/db/:user/:db/user/list",
            routing::get(routes::admin::db::user::list),
        )
        .route(
            "/admin/db/:user/:db/user/:other/add",
            routing::put(routes::admin::db::user::add),
        )
        .route(
            "/admin/db/:user/:db/user/:other/remove",
            routing::delete(routes::admin::db::user::remove),
        )
        .route("/db/list", routing::get(routes::db::list))
        .route("/db/:user/:db/add", routing::post(routes::db::add))
        .route("/db/:user/:db/audit", routing::get(routes::db::audit))
        .route("/db/:user/:db/backup", routing::post(routes::db::backup))
        .route("/db/:user/:db/clear", routing::post(routes::db::clear))
        .route("/db/:user/:db/convert", routing::post(routes::db::convert))
        .route("/db/:user/:db/copy", routing::post(routes::db::copy))
        .route("/db/:user/:db/delete", routing::delete(routes::db::delete))
        .route("/db/:user/:db/exec", routing::post(routes::db::exec))
        .route(
            "/db/:user/:db/optimize",
            routing::post(routes::db::optimize),
        )
        .route("/db/:user/:db/remove", routing::delete(routes::db::remove))
        .route("/db/:user/:db/rename", routing::post(routes::db::rename))
        .route("/db/:user/:db/restore", routing::post(routes::db::restore))
        .route(
            "/db/:user/:db/user/list",
            routing::get(routes::db::user::list),
        )
        .route(
            "/db/:user/:db/user/:other/add",
            routing::put(routes::db::user::add),
        )
        .route(
            "/db/:user/:db/user/:other/remove",
            routing::delete(routes::db::user::remove),
        )
        .route("/cluster/status", routing::get(routes::cluster::status))
        .route("/cluster", routing::post(routes::cluster::cluster))
        .route("/user/login", routing::post(routes::user::login))
        .route("/user/logout", routing::post(routes::user::logout))
        .route(
            "/user/change_password",
            routing::put(routes::user::change_password),
        )
        .route("/user/status", routing::get(routes::user::status));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);

    let router = Router::new()
        .merge(RapiDoc::with_openapi("/api/v1/openapi.json", Api::openapi()).path("/api/v1"))
        .nest("/api/v1", api_v1)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            logger::logger,
        ))
        .layer(cors)
        .with_state(state);

    if !basepath.is_empty() {
        Router::new().nest(&basepath, router)
    } else {
        router
    }
}
