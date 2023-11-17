use crate::logger;
use axum::body;
use axum::extract::State;
use axum::middleware;
use axum::routing;
use axum::Router;
use hyper::StatusCode;
use tokio::sync::broadcast::Sender;
use tower::ServiceBuilder;
use tower_http::map_request_body::MapRequestBodyLayer;

pub(crate) fn app(shutdown_sender: Sender<()>) -> Router {
    let logger = ServiceBuilder::new()
        .layer(MapRequestBodyLayer::new(body::boxed))
        .layer(middleware::from_fn(logger::logger));

    Router::new()
        .route("/", routing::get(root))
        .route("/error", routing::get(error))
        .route("/shutdown", routing::get(shutdown))
        .layer(logger)
        .with_state(shutdown_sender)
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
    use axum::body::Body;
    use axum::http::Request;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn hello_world() -> anyhow::Result<()> {
        let app = app(Sender::<()>::new(1));
        let request = Request::builder().uri("/").body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await?;
        assert_eq!(&body[..], b"Hello, World!");
        Ok(())
    }

    #[tokio::test]
    async fn error() -> anyhow::Result<()> {
        let app = app(Sender::<()>::new(1));
        let request = Request::builder().uri("/error").body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }

    #[tokio::test]
    async fn shutdown() -> anyhow::Result<()> {
        let (shutdown_sender, _shutdown_receiver) = tokio::sync::broadcast::channel::<()>(1);
        let app = app(shutdown_sender);
        let request = Request::builder().uri("/shutdown").body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(response.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn bad_shutdown() -> anyhow::Result<()> {
        let app = app(Sender::<()>::new(1));
        let request = Request::builder().uri("/shutdown").body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        Ok(())
    }
}
