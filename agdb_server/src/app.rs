use crate::db::DbPool;
use crate::logger;
use axum::body;
use axum::extract::FromRef;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing;
use axum::Router;
use serde::Deserialize;
use tokio::sync::broadcast::Sender;
use tower::ServiceBuilder;
use tower_http::map_request_body::MapRequestBodyLayer;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CreateDbType {
    File,
    Memory,
    MemoryMapped,
}

#[derive(Deserialize)]
struct CreateDb {
    name: String,
    db_type: CreateDbType,
}

#[derive(Clone)]
struct ServerState {
    db_pool: DbPool,
    shutdown_sender: Sender<()>,
}

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
        .route("/", routing::get(root))
        .route("/error", routing::get(error))
        .route("/shutdown", routing::get(shutdown))
        .route("/create_db", routing::get(create_db))
        .layer(logger)
        .with_state(state)
}

async fn create_db(State(_state): State<DbPool>, Query(request): Query<CreateDb>) -> StatusCode {
    println!("Creating db '{}' ({:?})", request.name, request.db_type);
    StatusCode::CREATED
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn error() -> StatusCode {
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
    use crate::db::DbType;
    use crate::db::ServerDb;
    use agdb::DbMemory;
    use axum::body::Body;
    use axum::http::Request;
    use axum::http::StatusCode;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::RwLock;
    use tower::ServiceExt;

    fn test_db_pool() -> anyhow::Result<DbPool> {
        Ok(DbPool(Arc::new(RwLock::new(DbPoolImpl {
            db: ServerDb {
                db: Arc::new(RwLock::new(DbType::Memory(DbMemory::new("test")?))),
            },
            pool: HashMap::new(),
        }))))
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
    async fn create_db() -> anyhow::Result<()> {
        let db_pool = test_db_pool()?;
        let app = app(Sender::<()>::new(1), db_pool);
        let request = Request::builder()
            .uri("/create_db?name=default&db_type=memory")
            .body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(response.status(), StatusCode::CREATED);
        Ok(())
    }
}
