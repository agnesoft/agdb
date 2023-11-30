use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;

#[derive(Debug)]
pub(crate) struct ServerError {
    pub(crate) description: String,
    pub(crate) status: StatusCode,
}

pub(crate) type ServerResult<T = ()> = Result<T, ServerError>;

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (self.status, self.description).into_response()
    }
}

impl<E: ToString> From<E> for ServerError {
    fn from(value: E) -> Self {
        Self {
            description: value.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ServerError {
    pub(crate) fn new(status: StatusCode, description: &str) -> Self {
        Self {
            description: description.to_string(),
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_response() {
        let error = ServerError::from("my error");
        let response = error.into_response();
        assert_eq!(response.status(), 500);
    }
}
