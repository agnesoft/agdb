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
            "/admin/db/:user/:db/backup",
            routing::post(routes::admin::db::backup),
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
        .route("/db/:user/:db/backup", routing::post(routes::db::backup))
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
        .route("/user/login", routing::post(routes::user::login))
        .route("/user/logout", routing::post(routes::user::logout))
        .route(
            "/user/change_password",
            routing::put(routes::user::change_password),
        );

    Router::new()
        .merge(RapiDoc::with_openapi("/api/v1/openapi.json", Api::openapi()).path("/api/v1"))
        .nest("/api/v1", api_v1)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            logger::logger,
        ))
        .with_state(state)
}
