use agdb::DbError;
use axum::routing;
use axum::Router;
use axum::Server;
use std::net::SocketAddr;

fn app() -> Router {
    Router::new().route("/", routing::get(root))
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() -> Result<(), DbError> {
    let app = app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
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

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await?;
        assert_eq!(&body[..], b"Hello, World!");
        Ok(())
    }
}
