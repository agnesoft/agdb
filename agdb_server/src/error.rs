use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;

pub(crate) struct ServerError {
    pub(crate) status: StatusCode,
    pub(crate) error: anyhow::Error,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (self.status, format!("{}", self.error)).into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: err.into(),
        }
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
