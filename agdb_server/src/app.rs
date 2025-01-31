use crate::api::Api;
use crate::cluster::Cluster;
use crate::config::Config;
use crate::db_pool::DbPool;
use crate::forward;
use crate::logger;
use crate::routes;
use crate::server_db::ServerDb;
use crate::server_error::ServerResult;
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
) -> ServerResult<Router> {
    #[cfg(feature = "studio")]
    routes::studio::init(&config)?;

    let basepath = config.basepath.clone();

    let state = ServerState {
        cluster,
        config,
        db_pool,
        server_db,
        shutdown_sender,
    };

    let api_v1 = Router::new()
        .route("/status", routing::get(routes::status))
        .route("/admin/shutdown", routing::post(routes::admin::shutdown))
        .route("/admin/status", routing::get(routes::admin::status))
        .route("/admin/user/list", routing::get(routes::admin::user::list))
        .route(
            "/admin/user/logout_all",
            routing::post(routes::admin::user::logout_all),
        )
        .route(
            "/admin/user/{username}/logout",
            routing::post(routes::admin::user::logout),
        )
        .route(
            "/admin/user/{username}/add",
            routing::post(routes::admin::user::add),
        )
        .route(
            "/admin/user/{username}/change_password",
            routing::put(routes::admin::user::change_password),
        )
        .route(
            "/admin/user/{username}/delete",
            routing::delete(routes::admin::user::delete),
        )
        .route("/admin/db/list", routing::get(routes::admin::db::list))
        .route(
            "/admin/db/{owner}/{db}/add",
            routing::post(routes::admin::db::add),
        )
        .route(
            "/admin/db/{owner}/{db}/audit",
            routing::get(routes::admin::db::audit),
        )
        .route(
            "/admin/db/{owner}/{db}/backup",
            routing::post(routes::admin::db::backup),
        )
        .route(
            "/admin/db/{owner}/{db}/clear",
            routing::post(routes::admin::db::clear),
        )
        .route(
            "/admin/db/{owner}/{db}/convert",
            routing::post(routes::admin::db::convert),
        )
        .route(
            "/admin/db/{owner}/{db}/copy",
            routing::post(routes::admin::db::copy),
        )
        .route(
            "/admin/db/{owner}/{db}/delete",
            routing::delete(routes::admin::db::delete),
        )
        .route(
            "/admin/db/{owner}/{db}/exec",
            routing::post(routes::admin::db::exec),
        )
        .route(
            "/admin/db/{owner}/{db}/exec_mut",
            routing::post(routes::admin::db::exec_mut),
        )
        .route(
            "/admin/db/{owner}/{db}/optimize",
            routing::post(routes::admin::db::optimize),
        )
        .route(
            "/admin/db/{owner}/{db}/remove",
            routing::delete(routes::admin::db::remove),
        )
        .route(
            "/admin/db/{owner}/{db}/rename",
            routing::post(routes::admin::db::rename),
        )
        .route(
            "/admin/db/{owner}/{db}/restore",
            routing::post(routes::admin::db::restore),
        )
        .route(
            "/admin/db/{owner}/{db}/user/list",
            routing::get(routes::admin::db::user::list),
        )
        .route(
            "/admin/db/{owner}/{db}/user/{username}/add",
            routing::put(routes::admin::db::user::add),
        )
        .route(
            "/admin/db/{owner}/{db}/user/{username}/remove",
            routing::delete(routes::admin::db::user::remove),
        )
        .route("/db/list", routing::get(routes::db::list))
        .route("/db/{owner}/{db}/add", routing::post(routes::db::add))
        .route("/db/{owner}/{db}/audit", routing::get(routes::db::audit))
        .route("/db/{owner}/{db}/backup", routing::post(routes::db::backup))
        .route("/db/{owner}/{db}/clear", routing::post(routes::db::clear))
        .route(
            "/db/{owner}/{db}/convert",
            routing::post(routes::db::convert),
        )
        .route("/db/{owner}/{db}/copy", routing::post(routes::db::copy))
        .route(
            "/db/{owner}/{db}/delete",
            routing::delete(routes::db::delete),
        )
        .route("/db/{owner}/{db}/exec", routing::post(routes::db::exec))
        .route(
            "/db/{owner}/{db}/exec_mut",
            routing::post(routes::db::exec_mut),
        )
        .route(
            "/db/{owner}/{db}/optimize",
            routing::post(routes::db::optimize),
        )
        .route(
            "/db/{owner}/{db}/remove",
            routing::delete(routes::db::remove),
        )
        .route("/db/{owner}/{db}/rename", routing::post(routes::db::rename))
        .route(
            "/db/{owner}/{db}/restore",
            routing::post(routes::db::restore),
        )
        .route(
            "/db/{owner}/{db}/user/list",
            routing::get(routes::db::user::list),
        )
        .route(
            "/db/{owner}/{db}/user/{username}/add",
            routing::put(routes::db::user::add),
        )
        .route(
            "/db/{owner}/{db}/user/{username}/remove",
            routing::delete(routes::db::user::remove),
        )
        .route("/cluster", routing::post(routes::cluster::cluster))
        .route("/cluster/user/login", routing::post(routes::cluster::login))
        .route(
            "/cluster/user/logout",
            routing::post(routes::cluster::logout),
        )
        .route(
            "/cluster/admin/user/{username}/logout",
            routing::post(routes::cluster::admin_logout),
        )
        .route(
            "/cluster/admin/user/logout_all",
            routing::post(routes::cluster::admin_logout_all),
        )
        .route("/cluster/status", routing::get(routes::cluster::status))
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

    #[cfg(feature = "studio")]
    let router = axum_extra::routing::RouterExt::route_with_tsr(
        Router::new(),
        "/studio/",
        routing::get(routes::studio::studio_root),
    )
    .route("/studio/{*file}", routing::get(routes::studio::studio));

    #[cfg(not(feature = "studio"))]
    let router = Router::new();

    let router = router
        .nest("/api/v1", api_v1)
        .merge(RapiDoc::with_openapi("/api/v1/openapi.json", Api::openapi()).path("/api/v1"))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            forward::forward_to_leader,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            logger::logger,
        ))
        .layer(cors)
        .with_state(state);

    Ok(if !basepath.is_empty() {
        Router::new().nest(&basepath, router)
    } else {
        router
    })
}
