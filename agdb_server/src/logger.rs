use crate::server_state::ServerState;
use crate::user_id::UserName;
use crate::utilities;
use agdb_api::LogLevelFilter;
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
use std::collections::HashMap;
use std::io::IsTerminal;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;
use std::time::Instant;
use std::time::SystemTime;

static LEVEL: AtomicU8 = AtomicU8::new(Level::Info as u8);

const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
#[allow(dead_code)]
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Level {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
}

impl From<u8> for Level {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::Error,
            2 => Self::Warn,
            3 => Self::Info,
            4 => Self::Debug,
            _ => Self::Info,
        }
    }
}

impl Level {
    fn colored_label(self) -> String {
        match self {
            Self::Error => colored_label(RED, "ERROR"),
            Self::Warn => colored_label(YELLOW, " WARN"),
            Self::Info => colored_label(GREEN, " INFO"),
            Self::Debug => colored_label(CYAN, "DEBUG"),
        }
    }
}

fn colors_enabled() -> bool {
    std::io::stdin().is_terminal()
}

fn colorize(color: &str, value: impl AsRef<str>) -> String {
    if colors_enabled() {
        format!("{color}{}{RESET}", value.as_ref())
    } else {
        value.as_ref().to_owned()
    }
}

fn colored_label(color: &str, value: &str) -> String {
    if colors_enabled() {
        format!("{color}{value}{RESET}")
    } else {
        value.to_owned()
    }
}

pub(crate) fn init(level: LogLevelFilter) {
    set_level(level);
}

pub(crate) fn set_level(level: LogLevelFilter) {
    let v = match level {
        LogLevelFilter::Off => 0,
        LogLevelFilter::Error => Level::Error as u8,
        LogLevelFilter::Warn => Level::Warn as u8,
        LogLevelFilter::Info => Level::Info as u8,
        LogLevelFilter::Debug | LogLevelFilter::Trace => Level::Debug as u8,
    };
    LEVEL.store(v, Ordering::Relaxed);
}

pub(crate) fn current_level() -> LogLevelFilter {
    match LEVEL.load(Ordering::Relaxed) {
        0 => LogLevelFilter::Off,
        1 => LogLevelFilter::Error,
        2 => LogLevelFilter::Warn,
        3 => LogLevelFilter::Info,
        4 => LogLevelFilter::Debug,
        _ => LogLevelFilter::Off,
    }
}

fn enabled(level: Level) -> bool {
    let current = LEVEL.load(Ordering::Relaxed);
    current >= level as u8
}

#[allow(dead_code)]
pub(crate) fn debug(message: &str) {
    if enabled(Level::Debug) {
        print_log(Level::Debug, message);
    }
}

pub(crate) fn info(message: &str) {
    if enabled(Level::Info) {
        print_log(Level::Info, message);
    }
}

pub(crate) fn warn(message: &str) {
    if enabled(Level::Warn) {
        print_log(Level::Warn, message);
    }
}

#[allow(dead_code)]
pub(crate) fn error(message: &str) {
    if enabled(Level::Error) {
        print_log(Level::Error, message);
    }
}

fn print_log(level: Level, message: &str) {
    let ts = utilities::timestamp();
    let label = level.colored_label();
    let ts = colorize(DIM, ts);
    println!("{ts} {label} {message}");
}

fn status_colored(status: u16) -> String {
    match status {
        ..=399 => colorize(GREEN, status.to_string()),
        400..=499 => colorize(YELLOW, status.to_string()),
        500.. => colorize(RED, status.to_string()),
    }
}

fn method_colored(method: &str) -> String {
    match method {
        "GET" => colorize(GREEN, method),
        _ => colorize(RED, method),
    }
}

struct LogRecord {
    node: usize,
    method: String,
    uri: String,
    user: String,
    status: u16,
    duration: u128,
    request_headers: HashMap<String, String>,
    request_body: String,
    response_headers: HashMap<String, String>,
    response_body: String,
}

impl LogRecord {
    fn print(&self, level: Level) {
        let lvl = level.colored_label();
        let status = status_colored(self.status);
        let method = method_colored(&self.method);
        let timestamp = utilities::format_system_time(&SystemTime::now());
        let user = if self.user.is_empty() {
            String::new()
        } else {
            format!(" [{}]", colorize(MAGENTA, &self.user))
        };
        let timestamp = colorize(DIM, timestamp);
        let duration = colorize(DIM, format!("{}μs", self.duration));
        let node = self.node;
        let uri = &self.uri;

        println!("{timestamp} {lvl} [{node}] {status} {method} {uri}{user} {duration}",);

        if level != Level::Info {
            if !self.request_headers.is_empty() {
                let headers_json = serde_json::to_string(&self.request_headers).unwrap_or_default();
                println!("  {} {headers_json}", colorize(DIM, "> Headers:"));
            }
            if !self.request_body.is_empty() {
                println!("  {} {}", colorize(DIM, "> Body:"), self.request_body);
            }
            if !self.response_headers.is_empty() {
                let headers_json =
                    serde_json::to_string(&self.response_headers).unwrap_or_default();
                println!("  {} {headers_json}", colorize(DIM, "< Headers:"));
            }
            if !self.response_body.is_empty() {
                println!("  {} {}", colorize(DIM, "< Body:"), self.response_body);
            }
        }
    }
}

pub(crate) async fn logger(
    state: State<ServerState>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    let level = LEVEL.load(Ordering::Relaxed);

    if level == 0 {
        return Ok(next.run(request).await);
    }

    let log_level = Level::from(level);

    let mut record = LogRecord {
        node: state.config.cluster_node_id,
        method: request.method().to_string(),
        uri: request.uri().to_string(),
        user: String::new(),
        status: 0,
        duration: 0,
        request_headers: HashMap::new(),
        request_body: String::new(),
        response_headers: HashMap::new(),
        response_body: String::new(),
    };

    let request = inspect_request(&mut record, log_level, &state, request).await?;

    let now = Instant::now();
    let response = next.run(request).await;
    record.duration = now.elapsed().as_micros();
    record.status = response.status().as_u16();

    if let Some(status_level) = should_log_status(log_level, &record.uri, record.status) {
        let show_details = state.config.log_body_limit != 0
            && (log_level == Level::Debug || status_level == Level::Error);
        let response = inspect_response(&mut record, response, show_details, &state).await?;
        record.print(status_level);
        return Ok(response);
    }

    Ok(response)
}

fn should_log_status(log_level: Level, uri: &str, status: u16) -> Option<Level> {
    if log_level < Level::Debug
        && (uri.ends_with("/api/v1/status")
            || uri.ends_with("/api/v1/cluster")
            || uri.ends_with("/api/v1/cluster/status"))
    {
        return None;
    }

    match status {
        500.. => Some(Level::Error),
        400..=499 if log_level >= Level::Warn => Some(Level::Warn),
        _ if log_level >= Level::Info => Some(Level::Info),
        _ => None,
    }
}

async fn inspect_request(
    record: &mut LogRecord,
    log_level: Level,
    state: &State<ServerState>,
    request: axum::http::Request<Body>,
) -> Result<axum::http::Request<Body>, axum::http::Response<Body>> {
    let (mut parts, mut body) = request.into_parts();

    record.user = parts
        .extract_with_state::<UserName, ServerState>(state)
        .await
        .unwrap_or_default()
        .0;

    if log_level >= Level::Debug && state.config.log_body_limit != 0 {
        let bytes = body.collect().await.map_err(map_error)?.to_bytes();
        let limit = bytes.len().min(state.config.log_body_limit as usize);
        record.request_body = String::from_utf8_lossy(&bytes[..limit]).into_owned();
        mask_password(&record.uri, &mut record.request_body);
        body = Body::from(bytes);

        record.request_headers = parts
            .headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
    }

    Ok(Request::from_parts(parts, body))
}

async fn inspect_response(
    record: &mut LogRecord,
    response: axum::http::Response<Body>,
    show_details: bool,
    state: &State<ServerState>,
) -> Result<axum::http::Response<Body>, axum::http::Response<Body>> {
    if !show_details {
        return Ok(response);
    }

    let (parts, mut body) = response.into_parts();

    let bytes = body.collect().await.map_err(map_error)?.to_bytes();
    let limit = bytes.len().min(state.config.log_body_limit as usize);
    record.response_body = String::from_utf8_lossy(&bytes[..limit]).into_owned();
    body = Body::from(bytes);

    record.response_headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    Ok(Response::from_parts(parts, body))
}

fn mask_password(uri: &str, body: &mut String) {
    if uri.contains("/login")
        || uri.contains("/change_password")
        || (uri.contains("/admin/user/") && uri.contains("/add"))
    {
        const PASSWORD_PATTERNS: [&str; 2] = ["\"password\"", "\"new_password\""];
        const QUOTE_PATTERN: &str = "\"";

        for pattern in PASSWORD_PATTERNS {
            if let Some(starting_index) = body.find(pattern)
                && let Some(start) = body[starting_index + pattern.len()..].find(QUOTE_PATTERN)
            {
                let mut skip = false;
                let start = starting_index + pattern.len() + start;
                let mut end = start + 1;

                for c in body[start + 1..].chars() {
                    end += c.len_utf8();

                    if skip {
                        skip = false;
                    } else if c == '\\' {
                        skip = true;
                    } else if c == '"' {
                        break;
                    }
                }

                *body = format!("{}\"***\"{}", &body[..start], &body[end..]);
            }
        }
    }
}

fn map_error(error: AxumError) -> Response {
    let mut response = Response::new(Body::from(error.to_string()));
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_ordering() {
        assert!(Level::Error < Level::Warn);
        assert!(Level::Warn < Level::Info);
        assert!(Level::Info < Level::Debug);
    }

    #[test]
    fn set_and_get_level() {
        set_level(LogLevelFilter::Warn);
        assert_eq!(current_level(), LogLevelFilter::Warn);
        set_level(LogLevelFilter::Info);
        assert_eq!(current_level(), LogLevelFilter::Info);
    }

    #[test]
    fn trace_maps_to_debug() {
        set_level(LogLevelFilter::Trace);
        assert_eq!(current_level(), LogLevelFilter::Debug);
    }

    #[test]
    fn off_level() {
        set_level(LogLevelFilter::Off);
        assert_eq!(current_level(), LogLevelFilter::Off);
        assert!(!enabled(Level::Error));
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
    fn mask_password_login() {
        let mut body = "\"password\":\"password\"".to_string();
        mask_password("/login", &mut body);
        assert_eq!(body, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_change_password() {
        let mut body = "\"password\":\"password\"".to_string();
        mask_password("/change_password", &mut body);
        assert_eq!(body, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_admin_user_add() {
        let mut body = "\"password\":\"password\"".to_string();
        mask_password("/admin/user/user1/add", &mut body);
        assert_eq!(body, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_exec() {
        let mut body = "\"password\":\"password\"".to_string();
        mask_password("/db/exec", &mut body);
        assert_eq!(body, "\"password\":\"password\"");
    }

    #[test]
    fn mask_password_spaces() {
        let mut body = "\"password\" : \" password \" ".to_string();
        mask_password("/login", &mut body);
        assert_eq!(body, "\"password\" : \"***\" ");
    }

    #[test]
    fn mask_password_quote_in_password() {
        let mut body = "\"password\":\" pass\\\"word \"".to_string();
        mask_password("/login", &mut body);
        assert_eq!(body, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_no_password() {
        let mut body = "\"body\":\"value\"".to_string();
        mask_password("/login", &mut body);
        assert_eq!(body, "\"body\":\"value\"");
    }

    #[test]
    fn mask_password_no_ending() {
        let mut body = "\"password\":\"value".to_string();
        mask_password("/login", &mut body);
        assert_eq!(body, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_no_value() {
        let mut body = "\"password\":\"".to_string();
        mask_password("/login", &mut body);
        assert_eq!(body, "\"password\":\"***\"");
    }

    #[test]
    fn mask_password_no_quote() {
        let mut body = "\"password\":".to_string();
        mask_password("/login", &mut body);
        assert_eq!(body, "\"password\":");
    }
}
