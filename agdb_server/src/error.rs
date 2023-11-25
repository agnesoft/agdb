use axum::response::IntoResponse;
use axum::response::Response;
use hyper::StatusCode;

pub(crate) struct ServerError(anyhow::Error);

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn into_response() {
        let error: ServerError = anyhow!("my error").into();
        let response = error.into_response();
        assert_eq!(response.status(), 500);
    }
}
