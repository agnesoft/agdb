use axum::body::BoxBody;
use axum::body::Full;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
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
    let request = request_log(request, &mut log_record).await?;
    let now = Instant::now();
    let response = next.run(request).await;
    log_record.time = now.elapsed().as_micros();
    let response = response_log(response, &mut log_record).await;
    print_log(log_record);
    response
}

fn print_log(log_record: LogRecord) {
    let message = serde_json::to_string(&log_record).unwrap_or_default();

    if log_record.status < 500 {
        tracing::info!(message);
    } else {
        tracing::error!(message);
    }
}

async fn request_log(
    request: Request<BoxBody>,
    log_record: &mut LogRecord,
) -> Result<Request<BoxBody>, Response> {
    log_record.method = request.method().to_string();
    log_record.uri = request.uri().to_string();
    log_record.version = format!("{:?}", request.version());
    log_record.request_headers = request
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let (parts, body) = request.into_parts();
    let bytes = hyper::body::to_bytes(body).await;
    let response = bytes
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    log_record.request_body = String::from_utf8_lossy(&response).to_string();

    Ok(Request::from_parts(
        parts,
        axum::body::boxed(Full::from(response)),
    ))
}

async fn response_log(
    response: Response<BoxBody>,
    log_record: &mut LogRecord,
) -> Result<impl IntoResponse, Response> {
    log_record.status = response.status().as_u16();
    log_record.response_headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let (parts, body) = response.into_parts();
    let bytes = hyper::body::to_bytes(body).await;
    let resposne = bytes
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;
    log_record.response = String::from_utf8_lossy(&resposne).to_string();

    Ok(Response::from_parts(
        parts,
        axum::body::boxed(Full::from(resposne)),
    ))
}
