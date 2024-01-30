use crate::server_state::ServerState;
use crate::user_id::UserName;
use axum::body::Body;
use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Error as AxumError;
use axum::RequestPartsExt;
use http_body_util::BodyExt;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Default, Serialize)]
struct LogRecord {
    method: String,
    version: String,
    user: String,
    uri: String,
    request_headers: HashMap<String, String>,
    request_body: String,
    status: u16,
    time: u128,
    response_headers: HashMap<String, String>,
    response: String,
}

impl LogRecord {
    fn print(&self) {
        let message = serde_json::to_string(&self).unwrap_or_default();

        match self.status {
            ..=399 => tracing::info!(message),
            400..=499 => tracing::warn!(message),
            500.. => tracing::error!(message),
        }
    }
}

pub(crate) async fn logger(
    state: State<ServerState>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    let mut log_record = LogRecord::default();
    let skip_body = request.uri().path().ends_with("openapi.json");
    let request = request_log(state, request, &mut log_record, skip_body).await?;
    let now = Instant::now();
    let response = next.run(request).await;
    log_record.time = now.elapsed().as_micros();
    let response = response_log(response, &mut log_record, skip_body).await;

    log_record.print();

    response
}

#[rustfmt::skip]
async fn request_log(
    state: State<ServerState>,
    request: Request,
    log_record: &mut LogRecord,
    skip_body: bool,
) -> Result<Request, Response> {
    log_record.method = request.method().to_string();
    log_record.uri = request.uri().to_string();
    log_record.version = format!("{:?}", request.version());
    log_record.request_headers = request
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    if !skip_body {
        let (mut parts, body) = request.into_parts();
        let bytes = body.collect().await.map_err(map_error)?.to_bytes();
        log_record.request_body = String::from_utf8_lossy(&bytes).to_string();
        log_record.user = parts.extract_with_state::<UserName, ServerState>(&state).await.unwrap_or_default().0;

        return Ok(Request::from_parts(parts, Body::from(bytes)));
    }

    Ok(request)
}

async fn response_log(
    response: Response,
    log_record: &mut LogRecord,
    skip_body: bool,
) -> Result<Response, Response> {
    log_record.status = response.status().as_u16();
    log_record.response_headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    if !skip_body {
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await.map_err(map_error)?.to_bytes();
        log_record.response = String::from_utf8_lossy(&bytes).to_string();

        return Ok(Response::from_parts(parts, Body::from(bytes)));
    }

    Ok(response)
}

fn map_error(error: AxumError) -> Response {
    let mut response = Response::new(Body::from(error.to_string()));
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn map_error_test() -> anyhow::Result<()> {
        let error = AxumError::new(anyhow::Error::msg("error"));
        let response = map_error(error);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = response.into_body().collect().await?.to_bytes();
        assert_eq!(&body[..], b"error");
        Ok(())
    }

    #[test]
    fn log_error_test() {
        let log_record = LogRecord {
            method: "GET".to_string(),
            uri: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            user: "user".to_string(),
            request_headers: HashMap::new(),
            request_body: String::new(),
            status: 500,
            time: 1,
            response_headers: HashMap::new(),
            response: String::new(),
        };
        log_record.print();
    }
}
