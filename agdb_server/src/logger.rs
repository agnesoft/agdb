use crate::server_state::ServerState;
use crate::user_id::UserName;
use axum::Error as AxumError;
use axum::RequestPartsExt;
use axum::body::Body;
use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use http_body_util::BodyExt;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Default, Serialize)]
struct LogRecord {
    node: usize,
    method: String,
    version: String,
    user: String,
    uri: String,
    request_headers: HashMap<String, String>,
    request: String,
    status: u16,
    time: u128,
    response_headers: HashMap<String, String>,
    response: String,
}

impl LogRecord {
    fn print(&self) {
        let message = serde_json::to_string(&self).unwrap_or_default();

        match self.status {
            ..=399 => {
                tracing::info!(message)
            }
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
    let path = request.uri().path();
    let skip_body = !path.contains("/api/v1/") || path.ends_with("openapi.json");
    let request = request_log(&state, request, &mut log_record, skip_body).await?;
    let now = Instant::now();
    let response = next.run(request).await;
    log_record.time = now.elapsed().as_micros();
    let response = response_log(&state, response, &mut log_record, skip_body).await;

    log_record.print();

    response
}

async fn request_log(
    state: &State<ServerState>,
    request: Request,
    log_record: &mut LogRecord,
    skip_body: bool,
) -> Result<Request, Response> {
    log_record.node = state.config.cluster_node_id;
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
        let log_bytes = if bytes.len() > state.config.log_body_limit as usize {
            &bytes[..state.config.log_body_limit as usize]
        } else {
            &bytes
        };
        log_record.request = String::from_utf8_lossy(log_bytes).to_string();

        mask_password(log_record);

        log_record.user = parts
            .extract_with_state::<UserName, ServerState>(state)
            .await
            .unwrap_or_default()
            .0;

        return Ok(Request::from_parts(parts, Body::from(bytes)));
    }

    Ok(request)
}

fn mask_password(log_record: &mut LogRecord) {
    if log_record.uri.contains("/login")
        || log_record.uri.contains("/change_password")
        || (log_record.uri.contains("/admin/user/") && log_record.uri.contains("/add"))
    {
        const PASSWORD_PATTERNS: [&str; 2] = ["\"password\"", "\"new_password\""];
        const QUOTE_PATTERN: &str = "\"";

        for pattern in PASSWORD_PATTERNS {
            if let Some(starting_index) = log_record.request.find(pattern)
                && let Some(start) =
                    log_record.request[starting_index + pattern.len()..].find(QUOTE_PATTERN)
                {
                    let mut skip = false;
                    let start = starting_index + pattern.len() + start;
                    let mut end = start + 1;

                    for c in log_record.request[start + 1..].chars() {
                        end += 1;

                        if skip {
                            skip = false;
                        } else if c == '\\' {
                            skip = true;
                        } else if c == '"' {
                            break;
                        }
                    }

                    log_record.request = format!(
                        "{}\"***\"{}",
                        &log_record.request[..start],
                        &log_record.request[end..]
                    );
                }
        }
    }
}

async fn response_log(
    state: &State<ServerState>,
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
        let log_bytes = if bytes.len() > state.config.log_body_limit as usize {
            &bytes[..state.config.log_body_limit as usize]
        } else {
            &bytes
        };
        log_record.response = String::from_utf8_lossy(log_bytes).to_string();

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

    fn log_record(uri: &str, request_body: &str) -> LogRecord {
        LogRecord {
            node: 0,
            method: "GET".to_string(),
            uri: uri.to_string(),
            version: "HTTP/1.1".to_string(),
            user: String::new(),
            request_headers: HashMap::new(),
            request: request_body.to_string(),
            status: StatusCode::OK.as_u16(),
            time: 0,
            response_headers: HashMap::new(),
            response: String::new(),
        }
    }

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
            node: 0,
            method: "GET".to_string(),
            uri: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            user: "user".to_string(),
            request_headers: HashMap::new(),
            request: String::new(),
            status: 500,
            time: 1,
            response_headers: HashMap::new(),
            response: String::new(),
        };
        log_record.print();
    }

    #[test]
    fn mask_password_login() {
        let mut record = log_record("/login", "\"password\":\"password\"");
        mask_password(&mut record);
        assert_eq!(record.request, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_change_password() {
        let mut record = log_record("/change_password", "\"password\":\"password\"");
        mask_password(&mut record);
        assert_eq!(record.request, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_admin_user_add() {
        let mut record = log_record("/admin/user/user1/add", "\"password\":\"password\"");
        mask_password(&mut record);
        assert_eq!(record.request, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_exec() {
        let mut record = log_record("/db/exec", "\"password\":\"password\"");
        mask_password(&mut record);
        assert_eq!(record.request, "\"password\":\"password\"");
    }

    #[test]
    fn mask_password_spaces() {
        let mut record = log_record("/login", "\"password\" : \" password \" ");
        mask_password(&mut record);
        assert_eq!(record.request, "\"password\" : \"***\" ");
    }

    #[test]
    fn mask_password_quote_in_password() {
        let mut record = log_record("/login", "\"password\":\" pass\\\"word \"");
        mask_password(&mut record);
        assert_eq!(record.request, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_no_password() {
        let mut record = log_record("/login", "\"body\":\"value\"");
        mask_password(&mut record);
        assert_eq!(record.request, "\"body\":\"value\"");
    }

    #[test]
    fn mask_password_no_ending() {
        let mut record = log_record("/login", "\"password\":\"value");
        mask_password(&mut record);
        assert_eq!(record.request, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_no_value() {
        let mut record = log_record("/login", "\"password\":\"");
        mask_password(&mut record);
        assert_eq!(record.request, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_no_quote() {
        let mut record = log_record("/login", "\"password\":");
        mask_password(&mut record);
        assert_eq!(record.request, "\"password\":");
    }
}
