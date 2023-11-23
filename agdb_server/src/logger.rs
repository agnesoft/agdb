use axum::body::BoxBody;
use axum::body::Full;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Error as AxumError;
use hyper::Request;
use hyper::StatusCode;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Default, Serialize)]
struct LogRecord {
    method: String,
    uri: String,
    version: String,
    request_headers: HashMap<String, String>,
    request_body: String,
    status: u16,
    time: u128,
    response_headers: HashMap<String, String>,
    response: String,
}

pub(crate) async fn logger(
    request: Request<BoxBody>,
    next: Next<BoxBody>,
) -> Result<impl IntoResponse, Response> {
    let mut log_record = LogRecord::default();
    let skip_body = request.uri().path().starts_with("/openapi");
    let request = request_log(request, &mut log_record, skip_body).await?;
    let now = Instant::now();
    let response = next.run(request).await;
    log_record.time = now.elapsed().as_micros();
    let response = response_log(response, &mut log_record, skip_body).await;
    let message = serde_json::to_string(&log_record).unwrap_or_default();

    match log_record.status {
        ..=399 => tracing::info!(message),
        400..=499 => tracing::warn!(message),
        500.. => tracing::error!(message),
    }

    response
}

async fn request_log(
    request: Request<BoxBody>,
    log_record: &mut LogRecord,
    skip_body: bool,
) -> Result<Request<BoxBody>, Response> {
    log_record.method = request.method().to_string();
    log_record.uri = request.uri().to_string();
    log_record.version = format!("{:?}", request.version());
    log_record.request_headers = request
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    if !skip_body {
        let (parts, body) = request.into_parts();
        let bytes = hyper::body::to_bytes(body).await.map_err(map_error)?;
        log_record.request_body = String::from_utf8_lossy(&bytes).to_string();

        return Ok(Request::from_parts(
            parts,
            axum::body::boxed(Full::from(bytes)),
        ));
    }

    Ok(request)
}

async fn response_log(
    response: Response<BoxBody>,
    log_record: &mut LogRecord,
    skip_body: bool,
) -> Result<impl IntoResponse, Response> {
    log_record.status = response.status().as_u16();
    log_record.response_headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    if !skip_body {
        let (parts, body) = response.into_parts();
        let resposne = hyper::body::to_bytes(body).await.map_err(map_error)?;
        log_record.response = String::from_utf8_lossy(&resposne).to_string();

        return Ok(Response::from_parts(
            parts,
            axum::body::boxed(Full::from(resposne)),
        ));
    }

    Ok(response)
}

fn map_error(error: AxumError) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn map_error_test() -> anyhow::Result<()> {
        let error = AxumError::new(anyhow::Error::msg("error"));
        let response = map_error(error);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = hyper::body::to_bytes(response.into_body()).await?;
        assert_eq!(&body[..], b"error");
        Ok(())
    }
}
