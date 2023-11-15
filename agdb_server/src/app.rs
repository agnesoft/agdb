use axum::extract::State;
use axum::routing;
use axum::Router;
use hyper::StatusCode;
use tokio::sync::broadcast::Sender;

pub(crate) fn app(shutdown_sender: Sender<()>) -> Router {
    Router::new().route("/", routing::get(root)).route(
        "/shutdown",
        routing::get(shutdown).with_state(shutdown_sender),
    )
}

async fn root() -> &'static str {
    "Hello, World!"
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
