use axum::routing;
use axum::Router;

pub(crate) fn app() -> Router {
    Router::new().route("/", routing::get(root))
}

async fn root() -> &'static str {
    "Hello, World!"
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
        let app = app();
        let request = Request::builder().uri("/").body(Body::empty())?;
        let response = app.oneshot(request).await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await?;
        assert_eq!(&body[..], b"Hello, World!");
        Ok(())
    }
}
