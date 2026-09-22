use crate::config::Config;
use crate::routes::ServerResult;
use crate::server_error::ServerError;
use axum::extract::Path;
use axum::response::IntoResponse;
use include_dir::Dir;
use include_dir::File;
use include_dir::include_dir;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::sync::OnceLock;

const DEFAULT_SERVER_HTTPS: &str = "https://localhost:3000";
const DEFAULT_SERVER_HTTP: &str = "http://localhost:3000";
static AGDB_STUDIO: Dir = include_dir!("agdb_studio/app/dist");
static STUDIO_FILES: OnceLock<HashMap<String, StudioFile>> = OnceLock::new();

struct StudioFile {
    content: Vec<u8>,
    content_type: &'static str,
}

fn all_files<'a>(dir: &'a Dir<'a>) -> Vec<&'a File<'a>> {
    let mut files: Vec<&File> = dir.files().collect();
    for subdir in dir.dirs() {
        files.extend(all_files(subdir));
    }
    files
}

fn content_type(path: &str) -> &'static str {
    if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else {
        "text/html"
    }
}

pub(crate) fn init(config: &Config) -> ServerResult {
    let mut files = HashMap::new();
    let server_url = config.server_url();
    let basepath = &config.basepath;

    for file in all_files(&AGDB_STUDIO) {
        let path = file
            .path()
            .to_str()
            .ok_or(init_error("Failed to read embedded file path"))?
            .to_string();

        let ct = content_type(&path);

        let content = if let Some(text) = file.contents_utf8() {
            let mut text = if text.contains(DEFAULT_SERVER_HTTPS) {
                text.replace(DEFAULT_SERVER_HTTPS, &server_url)
            } else {
                text.replace(DEFAULT_SERVER_HTTP, &server_url)
            };
            if !basepath.is_empty() {
                text = text.replace("\"/studio", &format!("\"{basepath}/studio"));
                text = text.replace("`/studio", &format!("`{basepath}/studio"));
                text = text.replace("'/studio", &format!("'{basepath}/studio"));
            }
            text.into_bytes()
        } else {
            file.contents().to_vec()
        };

        files.insert(
            path,
            StudioFile {
                content,
                content_type: ct,
            },
        );
    }

    STUDIO_FILES
        .set(files)
        .map_err(|_| init_error("STUDIO_FILES already initialized"))?;

    Ok(())
}

fn init_error(msg: &str) -> ServerError {
    ServerError::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
}

fn get_files() -> ServerResult<&'static HashMap<String, StudioFile>> {
    STUDIO_FILES.get().ok_or(ServerError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "studio files not initialized",
    ))
}

#[allow(clippy::type_complexity)]
fn serve_index() -> ServerResult<(StatusCode, [(&'static str, &'static str); 1], &'static [u8])> {
    let files = get_files()?;
    let index = files.get("index.html").ok_or(ServerError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "index.html not found",
    ))?;
    Ok((
        StatusCode::OK,
        [("Content-Type", "text/html")],
        &index.content,
    ))
}

pub(crate) async fn studio_root() -> ServerResult<impl IntoResponse> {
    serve_index()
}

pub(crate) async fn studio(Path(file): Path<String>) -> ServerResult<impl IntoResponse> {
    if file.ends_with("index.html") {
        return serve_index();
    }

    let files = get_files()?;

    if let Some(f) = files.get(&file) {
        return Ok((
            StatusCode::OK,
            [("Content-Type", f.content_type)],
            f.content.as_slice(),
        ));
    }

    serve_index()
}
