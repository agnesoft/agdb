use crate::config::Config;
use crate::routes::ServerResult;
use crate::server_error::ServerError;
use axum::extract::Path;
use axum::response::IntoResponse;
use include_dir::Dir;
use include_dir::include_dir;
use reqwest::StatusCode;
use std::sync::OnceLock;

static AGDB_STUDIO: Dir = include_dir!("agdb_studio/app/dist");
static AGDB_STUDIO_INDEX_HTML: OnceLock<String> = OnceLock::new();
static AGDB_STUDIO_INDEX_JS: OnceLock<String> = OnceLock::new();
static AGDB_STUDIO_INDEX_JS_CONTENT: OnceLock<String> = OnceLock::new();

fn init_error(msg: &str) -> ServerError {
    ServerError::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
}

fn init_index_js_name() -> ServerResult<String> {
    let index_html = AGDB_STUDIO
        .get_file("index.html")
        .ok_or(init_error("index.html not found"))?;
    let index_content = index_html
        .contents_utf8()
        .ok_or(init_error("index.html could not be read"))?;
    let (_, index_js_suffix) = index_content
        .split_once("src=\"/studio/assets/index")
        .ok_or(init_error("(src) Failed to find index.js in index.html"))?;
    let (index_js_suffix, _) = index_js_suffix
        .split_once(".js\"></script>")
        .ok_or(init_error("(script) Failed to find index.js in index.html"))?;

    let index_js_name = format!("assets/index{index_js_suffix}.js");

    AGDB_STUDIO_INDEX_JS.set(index_js_name.clone())?;

    Ok(index_js_name)
}

fn init_index_js_content(filename: &str, config: &Config) -> ServerResult {
    let index_js = AGDB_STUDIO
        .get_file(filename)
        .ok_or(init_error("3: Failed to find index.js in index.html"))?;
    let content = index_js.contents_utf8().expect("Failed to read index.js");
    let index_js_content = if !config.basepath.is_empty() {
        let f = content.replace("\"/studio", &format!("\"{}/studio", config.basepath));
        f.replace(
            "http://localhost:3000",
            &format!(
                "{}{}",
                config.address.trim_end_matches("/"),
                &config.basepath
            ),
        )
    } else {
        content.to_string()
    };

    AGDB_STUDIO_INDEX_JS_CONTENT.set(index_js_content)?;

    Ok(())
}

fn init_index_html(config: &Config) -> ServerResult {
    let content = AGDB_STUDIO
        .get_file("index.html")
        .ok_or(init_error("index.html not found"))?
        .contents_utf8()
        .ok_or(init_error("index.html could not be read"))?
        .replace("\"/studio/", &format!("\"{}/studio/", config.basepath));

    AGDB_STUDIO_INDEX_HTML.set(content)?;

    Ok(())
}

pub(crate) fn init(config: &Config) -> ServerResult {
    let index_js_name = init_index_js_name()?;
    init_index_js_content(&index_js_name, config)?;
    init_index_html(config)
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

    if AGDB_STUDIO_INDEX_JS.get() == Some(&file) {
        let content = AGDB_STUDIO_INDEX_JS_CONTENT.get().ok_or(ServerError::new(
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
