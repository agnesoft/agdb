use crate::config::Config;
use crate::routes::ServerResult;
use crate::server_error::ServerError;
use axum::extract::Path;
use axum::response::IntoResponse;
use include_dir::include_dir;
use include_dir::Dir;
use reqwest::StatusCode;
use std::sync::OnceLock;

static AGDB_STUDIO: Dir = include_dir!("agdb_studio/dist");
static AGDB_STUDIO_INDEX_JS: OnceLock<String> = OnceLock::new();
static AGDB_STUDIO_INDEX_HTML: OnceLock<String> = OnceLock::new();

pub(crate) fn init(config: &Config) {
    AGDB_STUDIO_INDEX_JS.get_or_init(|| {
        let index = AGDB_STUDIO
            .get_dir("assets")
            .expect("assets dir not found")
            .files()
            .find(|f| {
                f.path().extension().unwrap_or_default() == "js"
                    && f.path()
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .starts_with("index")
            })
            .expect("index.js not found")
            .contents_utf8()
            .expect("Failed to read index.js");

        let f = index.replace("\"/studio", &format!("\"{}/studio", config.basepath));

        if !config.basepath.is_empty() {
            f.replace(
                "http://localhost:3000",
                &format!(
                    "{}",
                    config
                        .address
                        .join(&config.basepath)
                        .expect("should be valid url and base path")
                ),
            )
        } else {
            f
        }
    });

    AGDB_STUDIO_INDEX_HTML.get_or_init(|| {
        AGDB_STUDIO
            .get_file("index.html")
            .expect("index.html not found")
            .contents_utf8()
            .expect("Failed to read index.html")
            .replace("\"/studio/", &format!("\"{}/studio/", config.basepath))
    });
}

async fn studio_index() -> ServerResult<(
    reqwest::StatusCode,
    [(&'static str, &'static str); 1],
    &'static [u8],
)> {
    let content = AGDB_STUDIO_INDEX_HTML.get().ok_or(ServerError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "index.html not found",
    ))?;

    Ok((
        StatusCode::OK,
        [("Content-Type", "text/html")],
        content.as_str().as_bytes(),
    ))
}

pub(crate) async fn studio_root() -> ServerResult<impl IntoResponse> {
    studio_index().await
}

pub(crate) async fn studio(Path(file): Path<String>) -> ServerResult<impl IntoResponse> {
    if file.ends_with("index.html") {
        return studio_index().await;
    }

    if file.ends_with(".js") && file.contains("index") {
        let content = AGDB_STUDIO_INDEX_JS.get().ok_or(ServerError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "index.js not found",
        ))?;

        return Ok((
            StatusCode::OK,
            [("Content-Type", "application/javascript")],
            content.as_str().as_bytes(),
        ));
    }

    let f = if let Some(f) = AGDB_STUDIO.get_file(&file) {
        f
    } else {
        return studio_index().await;
    };

    let content_type = if file.ends_with(".js") {
        "application/javascript"
    } else if file.ends_with(".css") {
        "text/css"
    } else if file.ends_with(".svg") {
        "image/svg+xml"
    } else if file.ends_with(".ico") {
        "image/x-icon"
    } else {
        "text/html"
    };

    Ok((
        StatusCode::OK,
        [("Content-Type", content_type)],
        f.contents(),
    ))
}
